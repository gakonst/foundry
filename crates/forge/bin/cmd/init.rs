use super::install::DependencyInstallOpts;
use clap::{Parser, ValueHint};
use eyre::Result;
use foundry_cli::utils::Git;
use foundry_common::fs;
use foundry_compilers::artifacts::remappings::Remapping;
use foundry_config::Config;
use std::path::{Path, PathBuf};
use yansi::Paint;

/// CLI arguments for `forge init`.
#[derive(Clone, Debug, Default, Parser)]
pub struct InitArgs {
    /// The root directory of the new project.
    #[arg(value_hint = ValueHint::DirPath, default_value = ".", value_name = "PATH")]
    pub root: PathBuf,

    /// The template to start from.
    #[arg(long, short)]
    pub template: Option<String>,

    /// Branch argument that can only be used with template option.
    /// If not specified, the default branch is used.
    #[arg(long, short, requires = "template")]
    pub branch: Option<String>,

    /// Do not install dependencies from the network.
    #[arg(long, conflicts_with = "template", visible_alias = "no-deps")]
    pub offline: bool,

    /// Create the project even if the specified root directory is not empty.
    #[arg(long, conflicts_with = "template")]
    pub force: bool,

    /// Create a .vscode/settings.json file with Solidity settings, and generate a remappings.txt
    /// file.
    #[arg(long, conflicts_with = "template")]
    pub vscode: bool,

    /// Use the parent git repository instead of initializing a new one.
    /// Only valid if the target is in a git repository.
    #[arg(long, conflicts_with = "template")]
    pub use_parent_git: bool,

    #[command(flatten)]
    pub install: DependencyInstallOpts,
}

impl InitArgs {
    pub fn run(self) -> Result<()> {
        let Self { root, template, branch, install, offline, force, vscode, use_parent_git } = self;
        let DependencyInstallOpts { shallow, no_git, no_commit } = install;

        // create the root dir if it does not exist
        if !root.exists() {
            fs::create_dir_all(&root)?;
        }
        let root = dunce::canonicalize(root)?;
        let git = Git::new(&root).shallow(shallow);

        // if a template is provided, then this command initializes a git repo,
        // fetches the template repo, and resets the git history to the head of the fetched
        // repo with no other history
        if let Some(template) = template {
            let template = if template.contains("://") {
                template
            } else {
                "https://github.com/".to_string() + &template
            };
            sh_println!("Initializing {} from {}...", root.display(), template)?;
            // initialize the git repository
            git.init()?;

            // fetch the template - always fetch shallow for templates since git history will be
            // collapsed. gitmodules will be initialized after the template is fetched
            git.fetch(true, &template, branch)?;

            if !no_commit {
                // reset git history to the head of the template
                // first get the commit hash that was fetched
                let commit_hash = git.commit_hash(true, "FETCH_HEAD")?;
                // format a commit message for the new repo
                let commit_msg = format!("chore: init from {template} at {commit_hash}");
                // get the hash of the FETCH_HEAD with the new commit message
                let new_commit_hash = git.commit_tree("FETCH_HEAD^{tree}", Some(commit_msg))?;
                // reset head of this repo to be the head of the template repo
                git.reset(true, new_commit_hash)?;
            }

            // if shallow, just initialize submodules
            if shallow {
                git.submodule_init()?;
            } else {
                // if not shallow, initialize and clone submodules (without fetching latest)
                git.submodule_update(false, false, true, true, std::iter::empty::<PathBuf>())?;
            }
        } else {
            // if target is not empty
            if root.read_dir().is_ok_and(|mut i| i.next().is_some()) {
                if !force {
                    eyre::bail!(
                        "Cannot run `init` on a non-empty directory.\n\
                        Run with the `--force` flag to initialize regardless."
                    );
                }
                sh_warn!("Target directory is not empty, but `--force` was specified")?;
            }

            // ensure git status is clean before generating anything
            if !no_git && !no_commit && !force && git.is_in_repo()? {
                git.ensure_clean()?;
            }

            sh_println!("Initializing {}...", root.display())?;

            // make the dirs
            let src = root.join("src");
            fs::create_dir_all(&src)?;

            let test = root.join("test");
            fs::create_dir_all(&test)?;

            let script = root.join("script");
            fs::create_dir_all(&script)?;

            // write the contract file
            let contract_path = src.join("Counter.sol");
            fs::write(contract_path, include_str!("../../assets/CounterTemplate.sol"))?;
            // write the tests
            let contract_path = test.join("Counter.t.sol");
            fs::write(contract_path, include_str!("../../assets/CounterTemplate.t.sol"))?;
            // write the script
            let contract_path = script.join("Counter.s.sol");
            fs::write(contract_path, include_str!("../../assets/CounterTemplate.s.sol"))?;
            // Write the default README file
            let readme_path = root.join("README.md");
            fs::write(readme_path, include_str!("../../assets/README.md"))?;

            // write foundry.toml, if it doesn't exist already
            let dest = root.join(Config::FILE_NAME);
            let mut config = Config::load_with_root(&root)?;
            if !dest.exists() {
                fs::write(dest, config.clone().into_basic().to_string_pretty()?)?;
            }
            let git = self.install.git(&config);

            // set up the repo
            if !no_git {
                init_git_repo(git, no_commit, use_parent_git)?;
            }

            // install forge-std
            if !offline {
                if root.join("lib/forge-std").exists() {
                    sh_warn!("\"lib/forge-std\" already exists, skipping install...")?;
                    self.install.install(&mut config, vec![])?;
                } else {
                    let dep = "https://github.com/foundry-rs/forge-std".parse()?;
                    self.install.install(&mut config, vec![dep])?;
                }
            }

            // init vscode settings
            if vscode {
                init_vscode(&root)?;
            }
        }

        sh_println!("{}", "    Initialized forge project".green())?;
        Ok(())
    }
}

/// Initialises `root` as a git repository, if it isn't one already, unless 'use_parent_git' is
/// true.
///
/// Creates `.gitignore` and `.github/workflows/test.yml`, if they don't exist already.
///
/// Commits everything if `no_commit` is false.
fn init_git_repo(git: Git<'_>, no_commit: bool, use_parent_git: bool) -> Result<()> {
    // git init
    // if not in a git repo, initialize one
    if !git.is_in_repo()? {
        git.init()?;
    } else {
        // if target is not the repo root init a new one unless `use_parent_git` is true
        if !git.is_repo_root()? && !use_parent_git {
            git.init()?;
        }
    }

    // .gitignore
    let gitignore = git.root.join(".gitignore");
    if !gitignore.exists() {
        fs::write(gitignore, include_str!("../../assets/.gitignoreTemplate"))?;
    }

    // github workflow
    let workflow = git.root.join(".github/workflows/test.yml");
    if !workflow.exists() {
        fs::create_dir_all(workflow.parent().unwrap())?;
        fs::write(workflow, include_str!("../../assets/workflowTemplate.yml"))?;
    }

    // commit everything
    if !no_commit {
        git.add(Some("--all"))?;
        git.commit("chore: forge init")?;
    }

    Ok(())
}

/// initializes the `.vscode/settings.json` file
fn init_vscode(root: &Path) -> Result<()> {
    let remappings_file = root.join("remappings.txt");
    if !remappings_file.exists() {
        let mut remappings = Remapping::find_many(&root.join("lib"))
            .into_iter()
            .map(|r| r.into_relative(root).to_relative_remapping().to_string())
            .collect::<Vec<_>>();
        if !remappings.is_empty() {
            remappings.sort();
            let content = remappings.join("\n");
            fs::write(remappings_file, content)?;
        }
    }

    let vscode_dir = root.join(".vscode");
    let settings_file = vscode_dir.join("settings.json");
    let mut settings = if !vscode_dir.is_dir() {
        fs::create_dir_all(&vscode_dir)?;
        serde_json::json!({})
    } else if settings_file.exists() {
        foundry_compilers::utils::read_json_file(&settings_file)?
    } else {
        serde_json::json!({})
    };

    let obj = settings.as_object_mut().expect("Expected settings object");
    // insert [vscode-solidity settings](https://github.com/juanfranblanco/vscode-solidity)
    let src_key = "solidity.packageDefaultDependenciesContractsDirectory";
    if !obj.contains_key(src_key) {
        obj.insert(src_key.to_string(), serde_json::Value::String("src".to_string()));
    }
    let lib_key = "solidity.packageDefaultDependenciesDirectory";
    if !obj.contains_key(lib_key) {
        obj.insert(lib_key.to_string(), serde_json::Value::String("lib".to_string()));
    }

    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(settings_file, content)?;

    Ok(())
}
