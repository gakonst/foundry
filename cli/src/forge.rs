use ethers::solc::{remappings::Remapping, Project, ProjectPathsConfig};

use structopt::StructOpt;

mod forge_opts;
use forge_opts::{Opts, Subcommands};

use crate::{
    cmd::Cmd,
    forge_opts::{Dependency, FullContractInfo},
};
use std::{process::Command, str::FromStr};

mod cmd;
mod utils;

#[tracing::instrument(err)]
fn main() -> eyre::Result<()> {
    utils::subscriber();

    let opts = Opts::from_args();
    match opts.sub {
        Subcommands::Test(cmd) => {
            let outcome = cmd.run()?;
            outcome.ensure_ok()?;
        }
        Subcommands::Build(cmd) => {
            cmd.run()?;
        }
        Subcommands::VerifyContract { contract, address, constructor_args } => {
            let FullContractInfo { path, name } = contract;
            let rt = tokio::runtime::Runtime::new().expect("could not start tokio rt");
            rt.block_on(cmd::verify::run(path, name, address, constructor_args))?;
        }
        Subcommands::Create { contract: _, verify: _ } => {
            unimplemented!("Not yet implemented")
        }
        Subcommands::Update { lib } => {
            // TODO: Should we add some sort of progress bar here? Would be nice
            // but not a requirement.
            // open the repo
            let repo = git2::Repository::open(".")?;

            // if a lib is specified, open it
            if let Some(lib) = lib {
                println!("Updating submodule {:?}", lib);
                repo.find_submodule(
                    &lib.into_os_string().into_string().expect("invalid submodule path"),
                )?
                .update(true, None)?;
            } else {
                Command::new("git")
                    .args(&["submodule", "update", "--init", "--recursive"])
                    .spawn()?
                    .wait()?;
            }
        }
        // TODO: Make it work with updates?
        Subcommands::Install { dependencies } => {
            install(std::env::current_dir()?, dependencies)?;
        }
        Subcommands::Remappings { lib_paths, root } => {
            let root = root.unwrap_or_else(|| std::env::current_dir().unwrap());
            let root = std::fs::canonicalize(root)?;

            let lib_paths = if lib_paths.is_empty() { vec![root.join("lib")] } else { lib_paths };
            let remappings: Vec<_> = lib_paths
                .iter()
                .map(|path| Remapping::find_many(&path).unwrap())
                .flatten()
                .collect();
            remappings.iter().for_each(|x| println!("{}", x));
        }
        Subcommands::Init { root, template } => {
            let root = root.unwrap_or_else(|| std::env::current_dir().unwrap());
            // create the root dir if it does not exist
            if !root.exists() {
                std::fs::create_dir_all(&root)?;
            }
            let root = std::fs::canonicalize(root)?;

            // if a template is provided, then this command is just an alias to `git clone <url>
            // <path>`
            if let Some(ref template) = template {
                println!("Initializing {} from {}...", root.display(), template);
                Command::new("git")
                    .args(&["clone", template, &root.display().to_string()])
                    .spawn()?
                    .wait()?;
            } else {
                println!("Initializing {}...", root.display());

                // make the dirs
                let src = root.join("src");
                let test = src.join("test");
                std::fs::create_dir_all(&test)?;
                let lib = root.join("lib");
                std::fs::create_dir(&lib)?;

                // write the contract file
                let contract_path = src.join("Contract.sol");
                std::fs::write(contract_path, include_str!("../../assets/ContractTemplate.sol"))?;
                // write the tests
                let contract_path = test.join("Contract.t.sol");
                std::fs::write(contract_path, include_str!("../../assets/ContractTemplate.t.sol"))?;

                // sets up git
                let is_git = Command::new("git")
                    .args(&["rev-parse,--is-inside-work-tree"])
                    .current_dir(&root)
                    .spawn()?
                    .wait()?;
                if !is_git.success() {
                    Command::new("git").arg("init").current_dir(&root).spawn()?.wait()?;
                }
                Command::new("git").args(&["add", "."]).current_dir(&root).spawn()?.wait()?;
                Command::new("git")
                    .args(&["commit", "-m", "chore: forge init"])
                    .current_dir(&root)
                    .spawn()?
                    .wait()?;

                Dependency::from_str("https://github.com/dapphub/ds-test")
                    .and_then(|dependency| install(root, vec![dependency]))?;
            }

            println!("Done.");
        }
        Subcommands::Completions { shell } => {
            Subcommands::clap().gen_completions_to("forge", shell, &mut std::io::stdout())
        }
        Subcommands::Clean { root } => {
            let root = root.unwrap_or_else(|| std::env::current_dir().unwrap());
            let paths = ProjectPathsConfig::builder().root(&root).build()?;
            let project = Project::builder().paths(paths).build()?;
            project.cleanup()?;
        }
        Subcommands::Snapshot(cmd) => {
            cmd.run()?;
        }
    }

    Ok(())
}

fn install(root: impl AsRef<std::path::Path>, dependencies: Vec<Dependency>) -> eyre::Result<()> {
    let libs = std::path::Path::new("lib");

    dependencies.iter().try_for_each(|dep| -> eyre::Result<_> {
        let path = libs.join(&dep.name);
        println!("Installing {} in {:?}, (url: {}, tag: {:?})", dep.name, path, dep.url, dep.tag);

        // install the dep
        Command::new("git")
            .args(&["submodule", "add", &dep.url, &path.display().to_string()])
            .current_dir(&root)
            .spawn()?
            .wait()?;

        // call update on it
        Command::new("git")
            .args(&["submodule", "update", "--init", "--recursive", &path.display().to_string()])
            .current_dir(&root)
            .spawn()?
            .wait()?;

        // checkout the tag if necessary
        let message = if let Some(ref tag) = dep.tag {
            Command::new("git")
                .args(&["checkout", "--recurse-submodules", tag])
                .current_dir(&path)
                .spawn()?
                .wait()?;

            Command::new("git").args(&["add", &path.display().to_string()]).spawn()?.wait()?;

            format!("forge install: {}\n\n{}", dep.name, tag)
        } else {
            format!("forge install: {}", dep.name)
        };

        Command::new("git").args(&["commit", "-m", &message]).current_dir(&root).spawn()?.wait()?;

        Ok(())
    })
}
