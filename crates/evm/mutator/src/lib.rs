use alloy_json_abi::{Function, JsonAbi as Abi};
use alloy_primitives::Bytes;
use eyre::{eyre, Result};
use foundry_cli::utils::FoundryPathExt;
use foundry_common::{ContractFilter, FunctionFilter, TestFunctionExt};
use foundry_compilers::{
    remappings::RelativeRemapping, Artifact, ArtifactId, ArtifactOutput, ProjectCompileOutput,
};
use gambit::{run_mutate, MutateParams};
use itertools::Itertools;
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

pub use gambit::Mutant;

/// Array of artifact ids, abi and bytecode
pub type GambitArtifacts = Vec<(ArtifactId, Abi, Bytes)>;

#[derive(Debug, Clone)]
pub struct MutatorConfigBuilder {
    solc: PathBuf,
    solc_allow_paths: Vec<PathBuf>,
    solc_include_paths: Vec<PathBuf>,
    solc_remappings: Vec<RelativeRemapping>,
    solc_optimize: bool,
}

impl MutatorConfigBuilder {
    pub fn new(
        solc: PathBuf,
        solc_optimize: bool,
        solc_allow_paths: Vec<PathBuf>,
        solc_include_paths: Vec<PathBuf>,
        solc_remappings: Vec<RelativeRemapping>,
    ) -> Self {
        Self { solc, solc_allow_paths, solc_include_paths, solc_remappings, solc_optimize }
    }

    pub fn build<A: ArtifactOutput>(
        self,
        src_folder_root: PathBuf,
        output: ProjectCompileOutput<A>,
    ) -> Result<Mutator> {
        // Converts the compiled output into artifactId and abi
        // It does not include files with .t.sol extension
        let artifacts: Vec<(ArtifactId, Abi, Bytes)> = output
            .into_artifacts()
            .filter_map(|(id, c)| match (id.source.as_path().is_sol_test(), c.into_parts()) {
                (false, (Some(abi), Some(bytecode), _)) => Some((id, abi, bytecode)),
                _ => None,
            })
            .collect::<Vec<(ArtifactId, Abi, Bytes)>>();

        let solc = self.solc.to_str().ok_or(eyre!("failed to decode solc root"))?;
        let solc_allow_paths: Vec<String> = self
            .solc_allow_paths
            .into_iter()
            .filter_map(|x| x.to_str().map(|x| x.to_string()))
            .collect();
        let solc_include_paths: String = self
            .solc_include_paths
            .into_iter()
            .filter_map(|x| x.to_str().map(|x| x.to_string()))
            .join(",");
        let solc_remappings: Vec<String> =
            self.solc_remappings.into_iter().map(|x| x.to_string()).collect();
        let source_root = src_folder_root.to_str().ok_or(eyre!("failed to decode source root"))?;

        Ok(Mutator::new(
            artifacts,
            source_root.to_owned(),
            solc.to_owned(),
            solc_allow_paths,
            solc_include_paths,
            solc_remappings,
            self.solc_optimize,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct Mutator {
    src_root: PathBuf,
    artifacts: GambitArtifacts,
    default_mutate_params: MutateParams,
}

impl Mutator {
    pub fn new(
        artifacts: GambitArtifacts,
        source_root: String,
        solc: String,
        solc_allow_paths: Vec<String>,
        solc_include_paths: String,
        solc_remappings: Vec<String>,
        solc_optimize: bool,
    ) -> Self {
        // create mutate params here
        let src_root = PathBuf::from(&source_root);
        let default_mutate_params = MutateParams {
            json: None,
            filename: None,
            num_mutants: None,
            random_seed: false,
            seed: 0,
            outdir: None,
            sourceroot: Some(source_root.into()),
            mutations: None,
            no_export: true,
            no_overwrite: false,
            solc: solc.into(),
            solc_optimize,
            functions: None,
            contract: None,
            solc_base_path: None,
            solc_allow_paths: Some(solc_allow_paths.into()),
            solc_include_path: Some(solc_include_paths.into()),
            solc_remappings: Some(solc_remappings.into()),
            skip_validate: false,
        };

        Self { src_root, artifacts, default_mutate_params }
    }

    /// Returns the number of matching functions
    pub fn matching_function_count<A: ContractFilter + FunctionFilter>(&self, filter: &A) -> usize {
        self.filtered_functions(filter).count()
    }

    /// Returns the name of the functions to generate Mutants
    pub fn get_artifact_functions<'a, A>(
        &'a self,
        filter: &'a A,
        abi: &'a Abi,
    ) -> impl Iterator<Item = String> + 'a
    where
        A: ContractFilter + FunctionFilter,
    {
        abi.functions()
            .filter_map(|func| filter.matches_function(&func.name).then_some(func.name.clone()))
    }

    /// Returns an iterator of functions matching filter
    pub fn filtered_functions<'a, A>(&'a self, filter: &'a A) -> impl Iterator<Item = &Function>
    where
        A: ContractFilter + FunctionFilter,
    {
        self.matching_artifacts(filter).flat_map(|(_, abi, _)| abi.functions())
    }

    /// Returns an iterator of function names matching filter
    pub fn get_function_names<'a, A>(&'a self, filter: &'a A) -> impl Iterator<Item = &String> + 'a
    where
        A: ContractFilter + FunctionFilter,
    {
        self.filtered_functions(filter)
            .filter_map(|func| filter.matches_function(&func.name).then_some(&func.name))
    }

    /// Returns mutation relevant artifacts matching the filter
    pub fn matching_artifacts<'a, A>(
        &'a self,
        filter: &'a A,
    ) -> impl Iterator<Item = &(ArtifactId, Abi, Bytes)>
    where
        A: ContractFilter + FunctionFilter,
    {
        self.artifacts.iter().filter(|(id, abi, _)| {
            id.source.starts_with(&self.src_root) &&
                !id.source.as_path().is_sol_test() &&
                filter.matches_path(&id.source) &&
                filter.matches_contract(&id.name) &&
                abi.functions().any(|func| filter.matches_function(&func.name))
        })
    }

    /// Returns all matching functions grouped by contract
    /// grouped by file (file -> contract -> functions)
    pub fn list<A: ContractFilter + FunctionFilter>(
        &self,
        filter: &A,
    ) -> BTreeMap<String, BTreeMap<String, Vec<String>>> {
        self.matching_artifacts(filter)
            .map(|(id, abi, _)| {
                let source = id.source.as_path().display().to_string();
                let name = id.name.clone();
                let functions = abi
                    .functions()
                    .filter(|func| !func.name.is_test())
                    .filter(|func| filter.matches_function(func.name.clone()))
                    .map(|func| func.name.clone())
                    .collect::<Vec<_>>();

                (source, name, functions)
            })
            .fold(BTreeMap::new(), |mut acc, (source, name, functions)| {
                acc.entry(source).or_default().insert(name, functions);
                acc
            })
    }

    /// Run mutation on contract functions that match configured filters
    /// @TODO we should support ability to disable writing out artifacts
    pub fn run_mutate<A>(
        self,
        _: bool,
        default_out_dir: PathBuf,
        filter: A,
    ) -> Result<HashMap<String, Vec<Mutant>>>
    where
        A: ContractFilter + FunctionFilter,
    {
        let mutant_params = self
            .matching_artifacts(&filter)
            .map(|(id, abi, _)| {
                let mut current_mutate_params = self.default_mutate_params.clone();
                let outdir = default_out_dir.join(id.name.clone());
                current_mutate_params.outdir = outdir.to_str().map(|x| x.to_owned());
                current_mutate_params.functions =
                    Some(self.get_artifact_functions(&filter, abi).collect_vec());
                current_mutate_params.filename =
                    Some(String::from(id.source.to_str().expect("failed run mutate filename")));
                current_mutate_params.contract = get_contract_name(&id.name);
                current_mutate_params
            })
            .collect_vec();

        run_mutate(mutant_params).map_err(|err| eyre!("{:?}", err))
    }
}

fn get_contract_name(name: &str) -> Option<String> {
    name.split(".").nth(0).map(|x| x.to_owned())
}