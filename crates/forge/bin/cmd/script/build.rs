use super::*;
use alloy_primitives::{Address, Bytes};
use eyre::{Context, ContextCompat, OptionExt, Result};
use forge::link::{link_with_nonce_or_address, LinkOutput};
use foundry_cli::utils::get_cached_entry_by_name;
use foundry_common::{
    compact_to_contract,
    compile::{self, ContractSources, ProjectCompiler},
    fs,
};
use foundry_compilers::{
    artifacts::{CompactContractBytecode, ContractBytecode, ContractBytecodeSome, Libraries},
    cache::SolFilesCache,
    contracts::ArtifactContracts,
    info::ContractInfo,
    ArtifactId, Project, ProjectCompileOutput,
};
use std::str::FromStr;

impl ScriptArgs {
    /// Compiles the file or project and the verify metadata.
    pub fn compile(&mut self, script_config: &mut ScriptConfig) -> Result<BuildOutput> {
        trace!(target: "script", "compiling script");

        self.build(script_config)
    }

    /// Compiles the file with auto-detection and compiler params.
    pub fn build(&mut self, script_config: &mut ScriptConfig) -> Result<BuildOutput> {
        let (project, output) = self.get_project_and_output(script_config)?;
        let output = output.with_stripped_file_prefixes(project.root());

        let mut sources: ContractSources = Default::default();

        let contracts = output
            .into_artifacts()
            .map(|(id, artifact)| -> Result<_> {
                // Sources are only required for the debugger, but it *might* mean that there's
                // something wrong with the build and/or artifacts.
                if let Some(source) = artifact.source_file() {
                    let path = source
                        .ast
                        .ok_or_else(|| eyre::eyre!("source from artifact has no AST"))?
                        .absolute_path;
                    let abs_path = project.root().join(path);
                    let source_code = fs::read_to_string(abs_path).wrap_err_with(|| {
                        format!("failed to read artifact source file for `{}`", id.identifier())
                    })?;
                    let contract = artifact.clone().into_contract_bytecode();
                    let source_contract = compact_to_contract(contract)?;
                    sources
                        .0
                        .entry(id.clone().name)
                        .or_default()
                        .insert(source.id, (source_code, source_contract));
                } else {
                    warn!(?id, "source not found");
                }
                Ok((id, artifact))
            })
            .collect::<Result<ArtifactContracts>>()?;

        let target = self.find_target(&project, &contracts)?.clone();
        script_config.target_contract = Some(target.clone());

        let libraries = script_config.config.solc_settings()?.libraries;

        let mut output = self.link_script_target(
            project,
            contracts,
            libraries,
            script_config.evm_opts.sender,
            script_config.sender_nonce,
            target,
        )?;

        output.sources = sources;

        Ok(output)
    }

    pub fn find_target<'a>(
        &self,
        project: &Project,
        contracts: &'a ArtifactContracts,
    ) -> Result<&'a ArtifactId> {
        let mut target_fname = dunce::canonicalize(&self.path)
            .wrap_err("Couldn't convert contract path to absolute path.")?
            .strip_prefix(project.root())
            .wrap_err("Couldn't strip project root from contract path.")?
            .to_str()
            .wrap_err("Bad path to string.")?
            .to_string();

        let no_target_name = if let Some(target_name) = &self.target_contract {
            target_fname = target_fname + ":" + target_name;
            false
        } else {
            true
        };

        let mut target = None;

        for (id, contract) in contracts.iter() {
            if no_target_name {
                // Match artifact source, and ignore interfaces
                if id.source == std::path::Path::new(&target_fname) &&
                    contract.bytecode.as_ref().map_or(false, |b| b.object.bytes_len() > 0)
                {
                    if target.is_some() {
                        eyre::bail!("Multiple contracts in the target path. Please specify the contract name with `--tc ContractName`")
                    }
                    target = Some(id);
                }
            } else {
                let (path, name) =
                    target_fname.rsplit_once(':').expect("The target specifier is malformed.");
                let path = std::path::Path::new(path);
                if path == id.source && name == id.name {
                    target = Some(id);
                }
            }
        }

        target.ok_or_eyre(format!("Could not find target contract: {}", target_fname))
    }

    pub fn link_script_target(
        &self,
        project: Project,
        contracts: ArtifactContracts,
        libraries: Libraries,
        sender: Address,
        nonce: u64,
        target: ArtifactId,
    ) -> Result<BuildOutput> {
        let LinkOutput {
            libs_to_deploy: predeploy_libraries,
            contracts: linked_contracts,
            libraries,
        } = link_with_nonce_or_address(&contracts, libraries, sender, nonce, &target)?;

        // Get linked target artifact
        let contract = linked_contracts
            .get(&target)
            .ok_or_eyre("Target contract not found in artifacts")?
            .clone();

        // Collect all linked contracts
        let highlevel_known_contracts = linked_contracts
            .iter()
            .filter_map(|(id, contract)| {
                ContractBytecodeSome::try_from(ContractBytecode::from(contract.clone()))
                    .ok()
                    .map(|tc| (id.clone(), tc))
            })
            .filter(|(_, tc)| !tc.bytecode.object.is_unlinked())
            .collect();

        Ok(BuildOutput {
            contract,
            known_contracts: contracts,
            highlevel_known_contracts,
            predeploy_libraries,
            sources: Default::default(),
            project,
            libraries,
        })
    }

    pub fn get_project_and_output(
        &mut self,
        script_config: &ScriptConfig,
    ) -> Result<(Project, ProjectCompileOutput)> {
        let project = script_config.config.project()?;

        let filters = self.opts.skip.clone().unwrap_or_default();
        // We received a valid file path.
        // If this file does not exist, `dunce::canonicalize` will
        // result in an error and it will be handled below.
        if let Ok(target_contract) = dunce::canonicalize(&self.path) {
            let output = compile::compile_target_with_filter(
                &target_contract,
                &project,
                self.opts.args.silent,
                self.verify,
                filters,
            )?;
            return Ok((project, output))
        }

        if !project.paths.has_input_files() {
            eyre::bail!("The project doesn't have any input files. Make sure the `script` directory is configured properly in foundry.toml. Otherwise, provide the path to the file.")
        }

        let contract = ContractInfo::from_str(&self.path)?;
        self.target_contract = Some(contract.name.clone());

        // We received `contract_path:contract_name`
        if let Some(path) = contract.path {
            let path =
                dunce::canonicalize(path).wrap_err("Could not canonicalize the target path")?;
            let output = compile::compile_target_with_filter(
                &path,
                &project,
                self.opts.args.silent,
                self.verify,
                filters,
            )?;
            self.path = path.to_string_lossy().to_string();
            return Ok((project, output))
        }

        // We received `contract_name`, and need to find its file path.
        let output = ProjectCompiler::new().compile(&project)?;
        let cache =
            SolFilesCache::read_joined(&project.paths).wrap_err("Could not open compiler cache")?;

        let (path, _) = get_cached_entry_by_name(&cache, &contract.name)
            .wrap_err("Could not find target contract in cache")?;
        self.path = path.to_string_lossy().to_string();

        Ok((project, output))
    }
}

pub struct BuildOutput {
    pub project: Project,
    pub contract: CompactContractBytecode,
    pub known_contracts: ArtifactContracts,
    pub highlevel_known_contracts: ArtifactContracts<ContractBytecodeSome>,
    pub libraries: Libraries,
    pub predeploy_libraries: Vec<Bytes>,
    pub sources: ContractSources,
}
