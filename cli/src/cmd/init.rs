//! init command

use crate::{
    cmd::{install::install, Cmd},
    opts::forge::Dependency,
    utils::p_println,
};
use clap::{Parser, ValueHint};
use foundry_config::Config;

use crate::cmd::install::DependencyInstallOpts;
use ansi_term::Colour;
use ethers::solc::remappings::Remapping;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
};

/// Command to initialize a new forge project
#[derive(Debug, Clone, Parser)]
pub struct InitArgs {
    #[clap(
    help = "the project's root path, default being the current working directory",
    value_hint = ValueHint::DirPath
    )]
    root: Option<PathBuf>,
    #[clap(help = "optional solidity template to start from", long, short)]
    template: Option<String>,
    #[clap(
        help = "initialize without creating a git repository",
        conflicts_with = "template",
        long
    )]
    no_git: bool,
    #[clap(help = "do not create initial commit", conflicts_with = "template", long)]
    no_commit: bool,
    #[clap(help = "do not print messages", short, long)]
    quiet: bool,
    #[clap(
        help = "run without installing libs from the network",
        conflicts_with = "template",
        long,
        alias = "no-deps"
    )]
    offline: bool,
    #[clap(help = "force init if project dir is not empty", conflicts_with = "template", long)]
    force: bool,
}

impl Cmd for InitArgs {
    type Output = ();

    fn run(self) -> eyre::Result<Self::Output> {
        let InitArgs { root, template, no_git, no_commit, quiet, offline, force } = self;

        let root = root.unwrap_or_else(|| std::env::current_dir().unwrap());
        // create the root dir if it does not exist
        if !root.exists() {
            std::fs::create_dir_all(&root)?;
        }
        let root = dunce::canonicalize(root)?;

        // if a template is provided, then this command is just an alias to `git clone <url>
        // <path>`
        if let Some(template) = template {
            let template = if template.starts_with("https://") {
                template
            } else {
                "https://github.com/".to_string() + &template
            };
            p_println!(!quiet => "Initializing {} from {}...", root.display(), template);
            Command::new("git")
                .args(&["clone", &template, &root.display().to_string()])
                .stdout(Stdio::piped())
                .spawn()?
                .wait()?;
        } else {
            // check if target is empty
            if !force && root.read_dir().map(|mut i| i.next().is_some()).unwrap_or(false) {
                eprintln!(
                    r#"{}: `forge init` cannot be run on a non-empty directory.

        run `forge init --force` to initialize regardless."#,
                    Colour::Red.paint("error")
                );
                std::process::exit(1);
            }

            p_println!(!quiet => "Initializing {}...", root.display());

            // make the dirs
            let src = root.join("src");
            let test = src.join("test");
            std::fs::create_dir_all(&test)?;

            // write the contract file
            let contract_path = src.join("Contract.sol");
            std::fs::write(contract_path, include_str!("../../../assets/ContractTemplate.sol"))?;
            // write the tests
            let contract_path = test.join("Contract.t.sol");
            std::fs::write(contract_path, include_str!("../../../assets/ContractTemplate.t.sol"))?;

            let dest = root.join(Config::FILE_NAME);
            if !dest.exists() {
                // write foundry.toml
                let mut config = Config::load_with_root(&root).into_basic();
                // add the ds-test remapping manually because we initialize before installing it
                if !offline {
                    config
                        .remappings
                        .push("ds-test/=lib/ds-test/src/".parse::<Remapping>().unwrap().into());
                }
                std::fs::write(dest, config.to_string_pretty()?)?;
            }

            // sets up git
            if !no_git {
                init_git_repo(&root, no_commit)?;
            }

            if !offline {
                let opts = DependencyInstallOpts { no_git, no_commit, quiet };
                Dependency::from_str("https://github.com/dapphub/ds-test")
                    .and_then(|dependency| install(&root, vec![dependency], opts))?;
            }
        }

        p_println!(!quiet => "    {} forge project.",   Colour::Green.paint("Initialized"));
        Ok(())
    }
}

/// initializes the root dir
fn init_git_repo(root: &Path, no_commit: bool) -> eyre::Result<()> {
    let is_git = Command::new("git")
        .args(&["rev-parse", "--is-inside-work-tree"])
        .current_dir(&root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait()?;

    if !is_git.success() {
        let gitignore_path = root.join(".gitignore");
        std::fs::write(gitignore_path, include_str!("../../../assets/.gitignoreTemplate"))?;

        Command::new("git")
            .arg("init")
            .current_dir(&root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait()?;

        if !no_commit {
            Command::new("git").args(&["add", "."]).current_dir(&root).spawn()?.wait()?;
            Command::new("git")
                .args(&["commit", "-m", "chore: forge init"])
                .current_dir(&root)
                .stdout(Stdio::piped())
                .spawn()?
                .wait()?;
        }
    }

    Ok(())
}
