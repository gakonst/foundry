//! Contains various tests for checking forge's commands

use crate::constants::*;
use foundry_compilers::artifacts::{remappings::Remapping, ConfigurableContractArtifact, Metadata};
use foundry_config::{
    parse_with_profile, BasicConfig, Chain, Config, FuzzConfig, InvariantConfig, SolidityErrorCode,
};
use foundry_test_utils::{
    foundry_compilers::PathStyle,
    rpc::next_mainnet_etherscan_api_key,
    snapbox::IntoData,
    util::{pretty_err, read_string, OutputExt, TestCommand},
};
use semver::Version;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    str::FromStr,
};

// tests `--help` is printed to std out
forgetest!(print_help, |_prj, cmd| {
    cmd.arg("--help").assert_success().stdout_eq(str![[r#"
Build, test, fuzz, debug and deploy Solidity contracts

Usage: forge[..] <COMMAND>

Commands:
...

Options:
  -h, --help
          Print help (see a summary with '-h')

  -j, --threads <THREADS>
          Number of threads to use. Specifying 0 defaults to the number of logical cores
          
          [aliases: jobs]

  -V, --version
          Print version

Display options:
      --color <COLOR>
          The color of the log messages

          Possible values:
          - auto:   Intelligently guess whether to use color output (default)
          - always: Force color output
          - never:  Force disable color output

      --json
          Format log messages as JSON

  -q, --quiet
          Do not print log messages

  -v, --verbosity...
          Verbosity level of the log messages.
          
          Pass multiple times to increase the verbosity (e.g. -v, -vv, -vvv).
          
          Depending on the context the verbosity levels have different meanings.
          
          For example, the verbosity levels of the EVM are:
          - 2 (-vv): Print logs for all tests.
          - 3 (-vvv): Print execution traces for failing tests.
          - 4 (-vvvv): Print execution traces for all tests, and setup traces for failing tests.
          - 5 (-vvvvv): Print execution and setup traces for all tests, including storage changes.

Find more information in the book: http://book.getfoundry.sh/reference/forge/forge.html

"#]]);
});

// checks that `clean` can be invoked even if out and cache don't exist
forgetest!(can_clean_non_existing, |prj, cmd| {
    cmd.arg("clean");
    cmd.assert_empty_stdout();
    prj.assert_cleaned();
});

// checks that `cache ls` can be invoked and displays the foundry cache
forgetest!(
    #[ignore]
    can_cache_ls,
    |_prj, cmd| {
        let chain = Chain::mainnet();
        let block1 = 100;
        let block2 = 101;

        let block1_cache_dir = Config::foundry_block_cache_dir(chain, block1).unwrap();
        let block1_file = Config::foundry_block_cache_file(chain, block1).unwrap();
        let block2_cache_dir = Config::foundry_block_cache_dir(chain, block2).unwrap();
        let block2_file = Config::foundry_block_cache_file(chain, block2).unwrap();
        let etherscan_cache_dir = Config::foundry_etherscan_chain_cache_dir(chain).unwrap();
        fs::create_dir_all(block1_cache_dir).unwrap();
        fs::write(block1_file, "{}").unwrap();
        fs::create_dir_all(block2_cache_dir).unwrap();
        fs::write(block2_file, "{}").unwrap();
        fs::create_dir_all(etherscan_cache_dir).unwrap();

        let output = cmd.args(["cache", "ls"]).assert_success().get_output().stdout_lossy();
        let output_lines = output.split('\n').collect::<Vec<_>>();
        println!("{output}");

        assert_eq!(output_lines.len(), 6);
        assert!(output_lines[0].starts_with("-️ mainnet ("));
        assert!(output_lines[1].starts_with("\t-️ Block Explorer ("));
        assert_eq!(output_lines[2], "");
        assert!(output_lines[3].starts_with("\t-️ Block 100 ("));
        assert!(output_lines[4].starts_with("\t-️ Block 101 ("));
        assert_eq!(output_lines[5], "");

        Config::clean_foundry_cache().unwrap();
    }
);

// checks that `cache clean` can be invoked and cleans the foundry cache
// this test is not isolated and modifies ~ so it is ignored
forgetest!(
    #[ignore]
    can_cache_clean,
    |_prj, cmd| {
        let cache_dir = Config::foundry_cache_dir().unwrap();
        let path = cache_dir.as_path();
        fs::create_dir_all(path).unwrap();
        cmd.args(["cache", "clean"]);
        cmd.assert_empty_stdout();

        assert!(!path.exists());
    }
);

// checks that `cache clean --etherscan` can be invoked and only cleans the foundry etherscan cache
// this test is not isolated and modifies ~ so it is ignored
forgetest!(
    #[ignore]
    can_cache_clean_etherscan,
    |_prj, cmd| {
        let cache_dir = Config::foundry_cache_dir().unwrap();
        let etherscan_cache_dir = Config::foundry_etherscan_cache_dir().unwrap();
        let path = cache_dir.as_path();
        let etherscan_path = etherscan_cache_dir.as_path();
        fs::create_dir_all(etherscan_path).unwrap();
        cmd.args(["cache", "clean", "--etherscan"]);
        cmd.assert_empty_stdout();

        assert!(path.exists());
        assert!(!etherscan_path.exists());

        Config::clean_foundry_cache().unwrap();
    }
);

// checks that `cache clean all --etherscan` can be invoked and only cleans the foundry etherscan
// cache. This test is not isolated and modifies ~ so it is ignored
forgetest!(
    #[ignore]
    can_cache_clean_all_etherscan,
    |_prj, cmd| {
        let rpc_cache_dir = Config::foundry_rpc_cache_dir().unwrap();
        let etherscan_cache_dir = Config::foundry_etherscan_cache_dir().unwrap();
        let rpc_path = rpc_cache_dir.as_path();
        let etherscan_path = etherscan_cache_dir.as_path();
        fs::create_dir_all(rpc_path).unwrap();
        fs::create_dir_all(etherscan_path).unwrap();
        cmd.args(["cache", "clean", "all", "--etherscan"]);
        cmd.assert_empty_stdout();

        assert!(rpc_path.exists());
        assert!(!etherscan_path.exists());

        Config::clean_foundry_cache().unwrap();
    }
);

// checks that `cache clean <chain>` can be invoked and cleans the chain cache
// this test is not isolated and modifies ~ so it is ignored
forgetest!(
    #[ignore]
    can_cache_clean_chain,
    |_prj, cmd| {
        let chain = Chain::mainnet();
        let cache_dir = Config::foundry_chain_cache_dir(chain).unwrap();
        let etherscan_cache_dir = Config::foundry_etherscan_chain_cache_dir(chain).unwrap();
        let path = cache_dir.as_path();
        let etherscan_path = etherscan_cache_dir.as_path();
        fs::create_dir_all(path).unwrap();
        fs::create_dir_all(etherscan_path).unwrap();
        cmd.args(["cache", "clean", "mainnet"]);
        cmd.assert_empty_stdout();

        assert!(!path.exists());
        assert!(!etherscan_path.exists());

        Config::clean_foundry_cache().unwrap();
    }
);

// checks that `cache clean <chain> --blocks 100,101` can be invoked and cleans the chain block
// caches this test is not isolated and modifies ~ so it is ignored
forgetest!(
    #[ignore]
    can_cache_clean_blocks,
    |_prj, cmd| {
        let chain = Chain::mainnet();
        let block1 = 100;
        let block2 = 101;
        let block3 = 102;
        let block1_cache_dir = Config::foundry_block_cache_dir(chain, block1).unwrap();
        let block2_cache_dir = Config::foundry_block_cache_dir(chain, block2).unwrap();
        let block3_cache_dir = Config::foundry_block_cache_dir(chain, block3).unwrap();
        let etherscan_cache_dir = Config::foundry_etherscan_chain_cache_dir(chain).unwrap();
        let block1_path = block1_cache_dir.as_path();
        let block2_path = block2_cache_dir.as_path();
        let block3_path = block3_cache_dir.as_path();
        let etherscan_path = etherscan_cache_dir.as_path();
        fs::create_dir_all(block1_path).unwrap();
        fs::create_dir_all(block2_path).unwrap();
        fs::create_dir_all(block3_path).unwrap();
        fs::create_dir_all(etherscan_path).unwrap();
        cmd.args(["cache", "clean", "mainnet", "--blocks", "100,101"]);
        cmd.assert_empty_stdout();

        assert!(!block1_path.exists());
        assert!(!block2_path.exists());
        assert!(block3_path.exists());
        assert!(etherscan_path.exists());

        Config::clean_foundry_cache().unwrap();
    }
);

// checks that `cache clean <chain> --etherscan` can be invoked and cleans the etherscan chain cache
// this test is not isolated and modifies ~ so it is ignored
forgetest!(
    #[ignore]
    can_cache_clean_chain_etherscan,
    |_prj, cmd| {
        let cache_dir = Config::foundry_chain_cache_dir(Chain::mainnet()).unwrap();
        let etherscan_cache_dir =
            Config::foundry_etherscan_chain_cache_dir(Chain::mainnet()).unwrap();
        let path = cache_dir.as_path();
        let etherscan_path = etherscan_cache_dir.as_path();
        fs::create_dir_all(path).unwrap();
        fs::create_dir_all(etherscan_path).unwrap();
        cmd.args(["cache", "clean", "mainnet", "--etherscan"]);
        cmd.assert_empty_stdout();

        assert!(path.exists());
        assert!(!etherscan_path.exists());

        Config::clean_foundry_cache().unwrap();
    }
);

// checks that init works
forgetest!(can_init_repo_with_config, |prj, cmd| {
    let foundry_toml = prj.root().join(Config::FILE_NAME);
    assert!(!foundry_toml.exists());

    cmd.args(["init", "--force"])
        .arg(prj.root())
        .assert_success()
        .stdout_eq(str![[r#"
Initializing [..]...
Installing forge-std in [..] (url: Some("https://github.com/foundry-rs/forge-std"), tag: None)
    Installed forge-std[..]
    Initialized forge project

"#]])
        .stderr_eq(str![[r#"
Warning: Target directory is not empty, but `--force` was specified
...

"#]]);

    let s = read_string(&foundry_toml);
    let _config: BasicConfig = parse_with_profile(&s).unwrap().unwrap().1;
});

// Checks that a forge project fails to initialise if dir is already git repo and dirty
forgetest!(can_detect_dirty_git_status_on_init, |prj, cmd| {
    prj.wipe();

    // initialize new git repo
    cmd.git_init();

    std::fs::write(prj.root().join("untracked.text"), "untracked").unwrap();

    // create nested dir and execute init in nested dir
    let nested = prj.root().join("nested");
    fs::create_dir_all(&nested).unwrap();

    cmd.current_dir(&nested);
    cmd.arg("init").assert_failure().stderr_eq(str![[r#"
Error: The target directory is a part of or on its own an already initialized git repository,
and it requires clean working and staging areas, including no untracked files.

Check the current git repository's status with `git status`.
Then, you can track files with `git add ...` and then commit them with `git commit`,
ignore them in the `.gitignore` file, or run this command again with the `--no-commit` flag.

"#]]);

    // ensure nothing was emitted, dir is empty
    assert!(!nested.read_dir().map(|mut i| i.next().is_some()).unwrap_or_default());
});

// Checks that a forge project can be initialized without creating a git repository
forgetest!(can_init_no_git, |prj, cmd| {
    prj.wipe();

    cmd.arg("init").arg(prj.root()).arg("--no-git").assert_success().stdout_eq(str![[r#"
Initializing [..]...
Installing forge-std in [..] (url: Some("https://github.com/foundry-rs/forge-std"), tag: None)
    Installed forge-std[..]
    Initialized forge project

"#]]);
    prj.assert_config_exists();

    assert!(!prj.root().join(".git").exists());
    assert!(prj.root().join("lib/forge-std").exists());
    assert!(!prj.root().join("lib/forge-std/.git").exists());
});

// Checks that quiet mode does not print anything
forgetest!(can_init_quiet, |prj, cmd| {
    prj.wipe();

    cmd.arg("init").arg(prj.root()).arg("-q").assert_empty_stdout();
});

// `forge init foobar` works with dir argument
forgetest!(can_init_with_dir, |prj, cmd| {
    prj.create_file("README.md", "non-empty dir");
    cmd.args(["init", "foobar"]);

    cmd.assert_success();
    assert!(prj.root().join("foobar").exists());
});

// `forge init foobar --template [template]` works with dir argument
forgetest!(can_init_with_dir_and_template, |prj, cmd| {
    cmd.args(["init", "foobar", "--template", "foundry-rs/forge-template"])
        .assert_success()
        .stdout_eq(str![[r#"
Initializing [..] from https://github.com/foundry-rs/forge-template...
    Initialized forge project

"#]]);

    assert!(prj.root().join("foobar/.git").exists());
    assert!(prj.root().join("foobar/foundry.toml").exists());
    assert!(prj.root().join("foobar/lib/forge-std").exists());
    // assert that gitmodules were correctly initialized
    assert!(prj.root().join("foobar/.git/modules").exists());
    assert!(prj.root().join("foobar/src").exists());
    assert!(prj.root().join("foobar/test").exists());
});

// `forge init foobar --template [template] --branch [branch]` works with dir argument
forgetest!(can_init_with_dir_and_template_and_branch, |prj, cmd| {
    cmd.args([
        "init",
        "foobar",
        "--template",
        "foundry-rs/forge-template",
        "--branch",
        "test/deployments",
    ])
    .assert_success()
    .stdout_eq(str![[r#"
Initializing [..] from https://github.com/foundry-rs/forge-template...
    Initialized forge project

"#]]);

    assert!(prj.root().join("foobar/.dapprc").exists());
    assert!(prj.root().join("foobar/lib/ds-test").exists());
    // assert that gitmodules were correctly initialized
    assert!(prj.root().join("foobar/.git/modules").exists());
    assert!(prj.root().join("foobar/src").exists());
    assert!(prj.root().join("foobar/scripts").exists());
});

// `forge init --force` works on non-empty dirs
forgetest!(can_init_non_empty, |prj, cmd| {
    prj.create_file("README.md", "non-empty dir");
    cmd.arg("init").arg(prj.root()).assert_failure().stderr_eq(str![[r#"
Error: Cannot run `init` on a non-empty directory.
Run with the `--force` flag to initialize regardless.

"#]]);

    cmd.arg("--force")
        .assert_success()
        .stdout_eq(str![[r#"
Initializing [..]...
Installing forge-std in [..] (url: Some("https://github.com/foundry-rs/forge-std"), tag: None)
    Installed forge-std[..]
    Initialized forge project

"#]])
        .stderr_eq(str![[r#"
Warning: Target directory is not empty, but `--force` was specified
...

"#]]);

    assert!(prj.root().join(".git").exists());
    assert!(prj.root().join("lib/forge-std").exists());
});

// `forge init --force` works on already initialized git repository
forgetest!(can_init_in_empty_repo, |prj, cmd| {
    let root = prj.root();

    // initialize new git repo
    let status = Command::new("git")
        .arg("init")
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("could not run git init");
    assert!(status.success());
    assert!(root.join(".git").exists());

    cmd.arg("init").arg(root).assert_failure().stderr_eq(str![[r#"
Error: Cannot run `init` on a non-empty directory.
Run with the `--force` flag to initialize regardless.

"#]]);

    cmd.arg("--force")
        .assert_success()
        .stdout_eq(str![[r#"
Initializing [..]...
Installing forge-std in [..] (url: Some("https://github.com/foundry-rs/forge-std"), tag: None)
    Installed forge-std[..]
    Initialized forge project

"#]])
        .stderr_eq(str![[r#"
Warning: Target directory is not empty, but `--force` was specified
...

"#]]);

    assert!(root.join("lib/forge-std").exists());
});

// `forge init --force` works on already initialized git repository
forgetest!(can_init_in_non_empty_repo, |prj, cmd| {
    let root = prj.root();

    // initialize new git repo
    let status = Command::new("git")
        .arg("init")
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("could not run git init");
    assert!(status.success());
    assert!(root.join(".git").exists());

    prj.create_file("README.md", "non-empty dir");
    prj.create_file(".gitignore", "not foundry .gitignore");

    cmd.arg("init").arg(root).assert_failure().stderr_eq(str![[r#"
Error: Cannot run `init` on a non-empty directory.
Run with the `--force` flag to initialize regardless.

"#]]);

    cmd.arg("--force")
        .assert_success()
        .stdout_eq(str![[r#"
Initializing [..]...
Installing forge-std in [..] (url: Some("https://github.com/foundry-rs/forge-std"), tag: None)
    Installed forge-std[..]
    Initialized forge project

"#]])
        .stderr_eq(str![[r#"
Warning: Target directory is not empty, but `--force` was specified
...

"#]]);

    assert!(root.join("lib/forge-std").exists());

    // not overwritten
    let gitignore = root.join(".gitignore");
    let gitignore = fs::read_to_string(gitignore).unwrap();
    assert_eq!(gitignore, "not foundry .gitignore");
});

// Checks that remappings.txt and .vscode/settings.json is generated
forgetest!(can_init_vscode, |prj, cmd| {
    prj.wipe();

    cmd.arg("init").arg(prj.root()).arg("--vscode").assert_success().stdout_eq(str![[r#"
Initializing [..]...
Installing forge-std in [..] (url: Some("https://github.com/foundry-rs/forge-std"), tag: None)
    Installed forge-std[..]
    Initialized forge project

"#]]);

    let settings = prj.root().join(".vscode/settings.json");
    assert!(settings.is_file());
    let settings: serde_json::Value = foundry_compilers::utils::read_json_file(&settings).unwrap();
    assert_eq!(
        settings,
        serde_json::json!({
            "solidity.packageDefaultDependenciesContractsDirectory": "src",
            "solidity.packageDefaultDependenciesDirectory": "lib"
        })
    );

    let remappings = prj.root().join("remappings.txt");
    assert!(remappings.is_file());
    let content = std::fs::read_to_string(remappings).unwrap();
    assert_eq!(content, "forge-std/=lib/forge-std/src/",);
});

// checks that forge can init with template
forgetest!(can_init_template, |prj, cmd| {
    prj.wipe();

    cmd.args(["init", "--template", "foundry-rs/forge-template"])
        .arg(prj.root())
        .assert_success()
        .stdout_eq(str![[r#"
Initializing [..] from https://github.com/foundry-rs/forge-template...
    Initialized forge project

"#]]);

    assert!(prj.root().join(".git").exists());
    assert!(prj.root().join("foundry.toml").exists());
    assert!(prj.root().join("lib/forge-std").exists());
    // assert that gitmodules were correctly initialized
    assert!(prj.root().join(".git/modules").exists());
    assert!(prj.root().join("src").exists());
    assert!(prj.root().join("test").exists());
});

// checks that forge can init with template and branch
forgetest!(can_init_template_with_branch, |prj, cmd| {
    prj.wipe();
    cmd.args(["init", "--template", "foundry-rs/forge-template", "--branch", "test/deployments"])
        .arg(prj.root())
        .assert_success()
        .stdout_eq(str![[r#"
Initializing [..] from https://github.com/foundry-rs/forge-template...
    Initialized forge project

"#]]);

    assert!(prj.root().join(".git").exists());
    assert!(prj.root().join(".dapprc").exists());
    assert!(prj.root().join("lib/ds-test").exists());
    // assert that gitmodules were correctly initialized
    assert!(prj.root().join(".git/modules").exists());
    assert!(prj.root().join("src").exists());
    assert!(prj.root().join("scripts").exists());
});

// checks that init fails when the provided template doesn't exist
forgetest!(fail_init_nonexistent_template, |prj, cmd| {
    prj.wipe();
    cmd.args(["init", "--template", "a"]).arg(prj.root()).assert_failure().stderr_eq(str![[r#"
remote: Not Found
fatal: repository 'https://github.com/a/' not found
Error: git fetch exited with code 128

"#]]);
});

// checks that `forge init --template [template] work with --no-commit
forgetest!(can_init_template_with_no_commit, |prj, cmd| {
    prj.wipe();
    cmd.args(["init", "--template", "foundry-rs/forge-template", "--no-commit"])
        .arg(prj.root())
        .assert_success()
        .stdout_eq(str![[r#"
Initializing [..] from https://github.com/foundry-rs/forge-template...
    Initialized forge project

"#]]);

    // show the latest commit message was not changed
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%s"]) // Get the latest commit message
        .output()
        .expect("Failed to execute git command");

    let commit_message = String::from_utf8_lossy(&output.stdout);
    assert!(
        !commit_message.starts_with("chore: init from foundry-rs/forge-template"),
        "Commit message should not start with 'chore: init from foundry-rs/forge-template'"
    );
});

// checks that clone works
forgetest!(can_clone, |prj, cmd| {
    prj.wipe();

    let foundry_toml = prj.root().join(Config::FILE_NAME);
    assert!(!foundry_toml.exists());

    cmd.args([
        "clone",
        "--etherscan-api-key",
        next_mainnet_etherscan_api_key().as_str(),
        "0x044b75f554b886A065b9567891e45c79542d7357",
    ])
    .arg(prj.root())
    .assert_success()
    .stdout_eq(str![[r#"
Downloading the source code of 0x044b75f554b886A065b9567891e45c79542d7357 from Etherscan...
Initializing [..]...
Installing forge-std in [..] (url: Some("https://github.com/foundry-rs/forge-std"), tag: None)
    Installed forge-std[..]
    Initialized forge project
Collecting the creation information of 0x044b75f554b886A065b9567891e45c79542d7357 from Etherscan...
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    let s = read_string(&foundry_toml);
    let _config: BasicConfig = parse_with_profile(&s).unwrap().unwrap().1;
});

// Checks that quiet mode does not print anything for clone
forgetest!(can_clone_quiet, |prj, cmd| {
    prj.wipe();

    cmd.args([
        "clone",
        "--etherscan-api-key",
        next_mainnet_etherscan_api_key().as_str(),
        "--quiet",
        "0xDb53f47aC61FE54F456A4eb3E09832D08Dd7BEec",
    ])
    .arg(prj.root())
    .assert_empty_stdout();
});

// checks that clone works with --no-remappings-txt
forgetest!(can_clone_no_remappings_txt, |prj, cmd| {
    prj.wipe();

    let foundry_toml = prj.root().join(Config::FILE_NAME);
    assert!(!foundry_toml.exists());

    cmd.args([
        "clone",
        "--etherscan-api-key",
        next_mainnet_etherscan_api_key().as_str(),
        "--no-remappings-txt",
        "0x33e690aEa97E4Ef25F0d140F1bf044d663091DAf",
    ])
    .arg(prj.root())
    .assert_success()
    .stdout_eq(str![[r#"
Downloading the source code of 0x33e690aEa97E4Ef25F0d140F1bf044d663091DAf from Etherscan...
Initializing [..]...
Installing forge-std in [..] (url: Some("https://github.com/foundry-rs/forge-std"), tag: None)
    Installed forge-std[..]
    Initialized forge project
Collecting the creation information of 0x33e690aEa97E4Ef25F0d140F1bf044d663091DAf from Etherscan...
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    let s = read_string(&foundry_toml);
    let _config: BasicConfig = parse_with_profile(&s).unwrap().unwrap().1;
});

// checks that clone works with --keep-directory-structure
forgetest!(can_clone_keep_directory_structure, |prj, cmd| {
    prj.wipe();

    let foundry_toml = prj.root().join(Config::FILE_NAME);
    assert!(!foundry_toml.exists());

    let output = cmd
        .forge_fuse()
        .args([
            "clone",
            "--etherscan-api-key",
            next_mainnet_etherscan_api_key().as_str(),
            "--keep-directory-structure",
            "0x33e690aEa97E4Ef25F0d140F1bf044d663091DAf",
        ])
        .arg(prj.root())
        .assert_success()
        .get_output()
        .stdout_lossy();

    if output.contains("502 Bad Gateway") {
        // etherscan nginx proxy issue, skip this test:
        //
        // stdout:
        // Downloading the source code of 0x33e690aEa97E4Ef25F0d140F1bf044d663091DAf from
        // Etherscan... 2024-07-05T11:40:11.801765Z ERROR etherscan: Failed to deserialize
        // response: expected value at line 1 column 1 res="<html>\r\n<head><title>502 Bad
        // Gateway</title></head>\r\n<body>\r\n<center><h1>502 Bad
        // Gateway</h1></center>\r\n<hr><center>nginx</center>\r\n</body>\r\n</html>\r\n"

        eprintln!("Skipping test due to 502 Bad Gateway");
        return;
    }

    let s = read_string(&foundry_toml);
    let _config: BasicConfig = parse_with_profile(&s).unwrap().unwrap().1;
});

// checks that `clean` removes dapptools style paths
forgetest!(can_clean, |prj, cmd| {
    prj.assert_create_dirs_exists();
    prj.assert_style_paths_exist(PathStyle::Dapptools);
    cmd.arg("clean");
    cmd.assert_empty_stdout();
    prj.assert_cleaned();
});

// checks that `clean` removes hardhat style paths
forgetest!(can_clean_hardhat, PathStyle::HardHat, |prj, cmd| {
    prj.assert_create_dirs_exists();
    prj.assert_style_paths_exist(PathStyle::HardHat);
    cmd.arg("clean");
    cmd.assert_empty_stdout();
    prj.assert_cleaned();
});

// checks that `clean` also works with the "out" value set in Config
forgetest_init!(can_clean_config, |prj, cmd| {
    prj.update_config(|config| config.out = "custom-out".into());
    cmd.arg("build").assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    // default test contract is written in custom out directory
    let artifact = prj.root().join(format!("custom-out/{TEMPLATE_TEST_CONTRACT_ARTIFACT_JSON}"));
    assert!(artifact.exists());

    cmd.forge_fuse().arg("clean").assert_empty_stdout();
    assert!(!artifact.exists());
});

// checks that `clean` removes fuzz and invariant cache dirs
forgetest_init!(can_clean_test_cache, |prj, cmd| {
    prj.update_config(|config| {
        config.fuzz = FuzzConfig::new("cache/fuzz".into());
        config.invariant = InvariantConfig::new("cache/invariant".into());
    });
    // default test contract is written in custom out directory
    let fuzz_cache_dir = prj.root().join("cache/fuzz");
    let _ = fs::create_dir(fuzz_cache_dir.clone());
    let invariant_cache_dir = prj.root().join("cache/invariant");
    let _ = fs::create_dir(invariant_cache_dir.clone());

    assert!(fuzz_cache_dir.exists());
    assert!(invariant_cache_dir.exists());

    cmd.forge_fuse().arg("clean").assert_empty_stdout();
    assert!(!fuzz_cache_dir.exists());
    assert!(!invariant_cache_dir.exists());
});

// checks that extra output works
forgetest_init!(can_emit_extra_output, |prj, cmd| {
    prj.clear();

    cmd.args(["build", "--extra-output", "metadata"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    let artifact_path = prj.paths().artifacts.join(TEMPLATE_CONTRACT_ARTIFACT_JSON);
    let artifact: ConfigurableContractArtifact =
        foundry_compilers::utils::read_json_file(&artifact_path).unwrap();
    assert!(artifact.metadata.is_some());

    cmd.forge_fuse()
        .args(["build", "--extra-output-files", "metadata", "--force"])
        .root_arg()
        .assert_success()
        .stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    let metadata_path =
        prj.paths().artifacts.join(format!("{TEMPLATE_CONTRACT_ARTIFACT_BASE}.metadata.json"));
    let _artifact: Metadata = foundry_compilers::utils::read_json_file(&metadata_path).unwrap();
});

// checks that extra output works
forgetest_init!(can_emit_multiple_extra_output, |prj, cmd| {
    cmd.args([
        "build",
        "--extra-output",
        "metadata",
        "legacyAssembly",
        "ir-optimized",
        "--extra-output",
        "ir",
    ])
    .assert_success()
    .stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    let artifact_path = prj.paths().artifacts.join(TEMPLATE_CONTRACT_ARTIFACT_JSON);
    let artifact: ConfigurableContractArtifact =
        foundry_compilers::utils::read_json_file(&artifact_path).unwrap();
    assert!(artifact.metadata.is_some());
    assert!(artifact.legacy_assembly.is_some());
    assert!(artifact.ir.is_some());
    assert!(artifact.ir_optimized.is_some());

    cmd.forge_fuse()
        .args([
            "build",
            "--extra-output-files",
            "metadata",
            "ir-optimized",
            "evm.bytecode.sourceMap",
            "evm.legacyAssembly",
            "--force",
        ])
        .root_arg()
        .assert_success()
        .stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    let metadata_path =
        prj.paths().artifacts.join(format!("{TEMPLATE_CONTRACT_ARTIFACT_BASE}.metadata.json"));
    let _artifact: Metadata = foundry_compilers::utils::read_json_file(&metadata_path).unwrap();

    let iropt = prj.paths().artifacts.join(format!("{TEMPLATE_CONTRACT_ARTIFACT_BASE}.iropt"));
    std::fs::read_to_string(iropt).unwrap();

    let sourcemap =
        prj.paths().artifacts.join(format!("{TEMPLATE_CONTRACT_ARTIFACT_BASE}.sourcemap"));
    std::fs::read_to_string(sourcemap).unwrap();

    let legacy_assembly = prj
        .paths()
        .artifacts
        .join(format!("{TEMPLATE_CONTRACT_ARTIFACT_BASE}.legacyAssembly.json"));
    std::fs::read_to_string(legacy_assembly).unwrap();
});

forgetest!(can_print_warnings, |prj, cmd| {
    prj.add_source(
        "Foo",
        r"
contract Greeter {
    function foo(uint256 a) public {
        uint256 x = 1;
    }
}
   ",
    )
    .unwrap();

    cmd.arg("build").assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful with warnings:
Warning (5667): Unused function parameter. Remove or comment out the variable name to silence this warning.
 [FILE]:5:18:
  |
5 |     function foo(uint256 a) public {
  |                  ^^^^^^^^^

Warning (2072): Unused local variable.
 [FILE]:6:9:
  |
6 |         uint256 x = 1;
  |         ^^^^^^^^^

Warning (2018): Function state mutability can be restricted to pure
 [FILE]:5:5:
  |
5 |     function foo(uint256 a) public {
  |     ^ (Relevant source part starts here and spans across multiple lines).


"#]]);
});

// Tests that direct import paths are handled correctly
forgetest!(can_handle_direct_imports_into_src, |prj, cmd| {
    prj.add_source(
        "Foo",
        r#"
import {FooLib} from "src/FooLib.sol";
struct Bar {
    uint8 x;
}
contract Foo {
    mapping(uint256 => Bar) bars;
    function checker(uint256 id) external {
        Bar memory b = bars[id];
        FooLib.check(b);
    }
    function checker2() external {
        FooLib.check2(this);
    }
}
   "#,
    )
    .unwrap();

    prj.add_source(
        "FooLib",
        r#"
import {Foo, Bar} from "src/Foo.sol";
library FooLib {
    function check(Bar memory b) public {}
    function check2(Foo f) public {}
}
   "#,
    )
    .unwrap();

    cmd.arg("build").assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);
});

// tests that the `inspect` command works correctly
forgetest!(can_execute_inspect_command, |prj, cmd| {
    let contract_name = "Foo";
    let path = prj
        .add_source(
            contract_name,
            r#"
contract Foo {
    event log_string(string);
    function run() external {
        emit log_string("script ran");
    }
}
    "#,
        )
        .unwrap();

    cmd.arg("inspect").arg(contract_name).arg("bytecode").assert_success().stdout_eq(str![[r#"
0x60806040[..]

"#]]);

    let info = format!("src/{}:{}", path.file_name().unwrap().to_string_lossy(), contract_name);
    cmd.forge_fuse().arg("inspect").arg(info).arg("bytecode").assert_success().stdout_eq(str![[
        r#"
0x60806040[..]

"#
    ]]);
});

// test that `forge snapshot` commands work
forgetest!(can_check_snapshot, |prj, cmd| {
    prj.insert_ds_test();

    prj.add_source(
        "ATest.t.sol",
        r#"
import "./test.sol";
contract ATest is DSTest {
    function testExample() public {
        assertTrue(true);
    }
}
   "#,
    )
    .unwrap();

    cmd.args(["snapshot"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

Ran 1 test for src/ATest.t.sol:ATest
[PASS] testExample() ([GAS])
Suite result: ok. 1 passed; 0 failed; 0 skipped; [ELAPSED]

Ran 1 test suite [ELAPSED]: 1 tests passed, 0 failed, 0 skipped (1 total tests)

"#]]);

    cmd.arg("--check").assert_success().stdout_eq(str![[r#"
No files changed, compilation skipped

Ran 1 test for src/ATest.t.sol:ATest
[PASS] testExample() ([GAS])
Suite result: ok. 1 passed; 0 failed; 0 skipped; [ELAPSED]

Ran 1 test suite [ELAPSED]: 1 tests passed, 0 failed, 0 skipped (1 total tests)

"#]]);
});

// test that `forge build` does not print `(with warnings)` if file path is ignored
forgetest!(can_compile_without_warnings_ignored_file_paths, |prj, cmd| {
    // Ignoring path and setting empty error_codes as default would set would set some error codes
    prj.update_config(|config| {
        config.ignored_file_paths = vec![Path::new("src").to_path_buf()];
        config.ignored_error_codes = vec![];
    });

    prj.add_raw_source(
        "src/example.sol",
        r"
pragma solidity *;
contract A {
    function testExample() public {}
}
",
    )
    .unwrap();

    cmd.args(["build", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    // Reconfigure without ignored paths or error codes and check for warnings
    prj.update_config(|config| config.ignored_file_paths = vec![]);

    cmd.forge_fuse().args(["build", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful with warnings:
Warning (1878): SPDX license identifier not provided in source file. Before publishing, consider adding a comment containing "SPDX-License-Identifier: <SPDX-License>" to each source file. Use "SPDX-License-Identifier: UNLICENSED" for non-open-source code. Please see https://spdx.org for more information.
Warning: SPDX license identifier not provided in source file. Before publishing, consider adding a comment containing "SPDX-License-Identifier: <SPDX-License>" to each source file. Use "SPDX-License-Identifier: UNLICENSED" for non-open-source code. Please see https://spdx.org for more information.
[FILE]


"#]]);
});

// test that `forge build` does not print `(with warnings)` if there aren't any
forgetest!(can_compile_without_warnings, |prj, cmd| {
    prj.update_config(|config| {
        config.ignored_error_codes = vec![SolidityErrorCode::SpdxLicenseNotProvided];
    });
    prj.add_raw_source(
        "A",
        r"
pragma solidity *;
contract A {
    function testExample() public {}
}
   ",
    )
    .unwrap();

    cmd.args(["build", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    // don't ignore errors
    prj.update_config(|config| {
        config.ignored_error_codes = vec![];
    });

    cmd.forge_fuse().args(["build", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful with warnings:
Warning (1878): SPDX license identifier not provided in source file. Before publishing, consider adding a comment containing "SPDX-License-Identifier: <SPDX-License>" to each source file. Use "SPDX-License-Identifier: UNLICENSED" for non-open-source code. Please see https://spdx.org for more information.
Warning: SPDX license identifier not provided in source file. Before publishing, consider adding a comment containing "SPDX-License-Identifier: <SPDX-License>" to each source file. Use "SPDX-License-Identifier: UNLICENSED" for non-open-source code. Please see https://spdx.org for more information.
[FILE]


"#]]);
});

// test that `forge build` compiles when severity set to error, fails when set to warning, and
// handles ignored error codes as an exception
forgetest!(can_fail_compile_with_warnings, |prj, cmd| {
    prj.update_config(|config| {
        config.ignored_error_codes = vec![];
        config.deny_warnings = false;
    });
    prj.add_raw_source(
        "A",
        r"
pragma solidity *;
contract A {
    function testExample() public {}
}
   ",
    )
    .unwrap();

    // there are no errors
    cmd.args(["build", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful with warnings:
Warning (1878): SPDX license identifier not provided in source file. Before publishing, consider adding a comment containing "SPDX-License-Identifier: <SPDX-License>" to each source file. Use "SPDX-License-Identifier: UNLICENSED" for non-open-source code. Please see https://spdx.org for more information.
Warning: SPDX license identifier not provided in source file. Before publishing, consider adding a comment containing "SPDX-License-Identifier: <SPDX-License>" to each source file. Use "SPDX-License-Identifier: UNLICENSED" for non-open-source code. Please see https://spdx.org for more information.
[FILE]


"#]]);

    // warning fails to compile
    prj.update_config(|config| {
        config.ignored_error_codes = vec![];
        config.deny_warnings = true;
    });

    cmd.forge_fuse().args(["build", "--force"]).assert_failure().stderr_eq(str![[r#"
Error: Compiler run failed:
Warning (1878): SPDX license identifier not provided in source file. Before publishing, consider adding a comment containing "SPDX-License-Identifier: <SPDX-License>" to each source file. Use "SPDX-License-Identifier: UNLICENSED" for non-open-source code. Please see https://spdx.org for more information.
Warning: SPDX license identifier not provided in source file. Before publishing, consider adding a comment containing "SPDX-License-Identifier: <SPDX-License>" to each source file. Use "SPDX-License-Identifier: UNLICENSED" for non-open-source code. Please see https://spdx.org for more information.
[FILE]

"#]]);

    // ignores error code and compiles
    prj.update_config(|config| {
        config.ignored_error_codes = vec![SolidityErrorCode::SpdxLicenseNotProvided];
        config.deny_warnings = true;
    });

    cmd.forge_fuse().args(["build", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);
});

// test that a failing `forge build` does not impact followup builds
forgetest!(can_build_after_failure, |prj, cmd| {
    prj.insert_ds_test();

    prj.add_source(
        "ATest.t.sol",
        r#"
import "./test.sol";
contract ATest is DSTest {
    function testExample() public {
        assertTrue(true);
    }
}
   "#,
    )
    .unwrap();
    prj.add_source(
        "BTest.t.sol",
        r#"
import "./test.sol";
contract BTest is DSTest {
    function testExample() public {
        assertTrue(true);
    }
}
   "#,
    )
    .unwrap();

    cmd.arg("build").assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
...
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    prj.assert_cache_exists();
    prj.assert_artifacts_dir_exists();

    let syntax_err = r#"
import "./test.sol";
contract CTest is DSTest {
    function testExample() public {
        THIS WILL CAUSE AN ERROR
    }
}
   "#;

    // introduce contract with syntax error
    prj.add_source("CTest.t.sol", syntax_err).unwrap();

    // `forge build --force` which should fail
    cmd.forge_fuse().args(["build", "--force"]).assert_failure().stderr_eq(str![[r#"
Error: Compiler run failed:
Error (2314): Expected ';' but got identifier
 [FILE]:7:19:
  |
7 |         THIS WILL CAUSE AN ERROR
  |                   ^^^^^

"#]]);

    // but ensure this cleaned cache and artifacts
    assert!(!prj.paths().artifacts.exists());
    assert!(!prj.cache().exists());

    // still errors
    cmd.forge_fuse().args(["build", "--force"]).assert_failure().stderr_eq(str![[r#"
Error: Compiler run failed:
Error (2314): Expected ';' but got identifier
 [FILE]:7:19:
  |
7 |         THIS WILL CAUSE AN ERROR
  |                   ^^^^^

"#]]);

    // resolve the error by replacing the file
    prj.add_source(
        "CTest.t.sol",
        r#"
import "./test.sol";
contract CTest is DSTest {
    function testExample() public {
         assertTrue(true);
    }
}
   "#,
    )
    .unwrap();

    cmd.forge_fuse().args(["build", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    prj.assert_cache_exists();
    prj.assert_artifacts_dir_exists();

    // ensure cache is unchanged after error
    let cache = fs::read_to_string(prj.cache()).unwrap();

    // introduce the error again but building without force
    prj.add_source("CTest.t.sol", syntax_err).unwrap();
    cmd.forge_fuse().arg("build").assert_failure().stderr_eq(str![[r#"
Error: Compiler run failed:
Error (2314): Expected ';' but got identifier
 [FILE]:7:19:
  |
7 |         THIS WILL CAUSE AN ERROR
  |                   ^^^^^

"#]]);

    // ensure unchanged cache file
    let cache_after = fs::read_to_string(prj.cache()).unwrap();
    assert_eq!(cache, cache_after);
});

// test to check that install/remove works properly
forgetest!(can_install_and_remove, |prj, cmd| {
    cmd.git_init();

    let libs = prj.root().join("lib");
    let git_mod = prj.root().join(".git/modules/lib");
    let git_mod_file = prj.root().join(".gitmodules");

    let forge_std = libs.join("forge-std");
    let forge_std_mod = git_mod.join("forge-std");

    let install = |cmd: &mut TestCommand| {
        cmd.forge_fuse()
            .args(["install", "foundry-rs/forge-std", "--no-commit"])
            .assert_success()
            .stdout_eq(str![[r#"
Installing forge-std in [..] (url: Some("https://github.com/foundry-rs/forge-std"), tag: None)
    Installed forge-std[..]

"#]]);

        assert!(forge_std.exists());
        assert!(forge_std_mod.exists());

        let submods = read_string(&git_mod_file);
        assert!(submods.contains("https://github.com/foundry-rs/forge-std"));
    };

    let remove = |cmd: &mut TestCommand, target: &str| {
        // TODO: flaky behavior with URL, sometimes it is None, sometimes it is Some("https://github.com/lib/forge-std")
        cmd.forge_fuse().args(["remove", "--force", target]).assert_success().stdout_eq(str![[
            r#"
Removing 'forge-std' in [..], (url: [..], tag: None)

"#
        ]]);

        assert!(!forge_std.exists());
        assert!(!forge_std_mod.exists());
        let submods = read_string(&git_mod_file);
        assert!(!submods.contains("https://github.com/foundry-rs/forge-std"));
    };

    install(&mut cmd);
    remove(&mut cmd, "forge-std");

    // install again and remove via relative path
    install(&mut cmd);
    remove(&mut cmd, "lib/forge-std");
});

// test to check we can run `forge install` in an empty dir <https://github.com/foundry-rs/foundry/issues/6519>
forgetest!(can_install_empty, |prj, cmd| {
    // create
    cmd.git_init();
    cmd.forge_fuse().args(["install"]);
    cmd.assert_empty_stdout();

    // create initial commit
    fs::write(prj.root().join("README.md"), "Initial commit").unwrap();

    cmd.git_add();
    cmd.git_commit("Initial commit");

    cmd.forge_fuse().args(["install"]);
    cmd.assert_empty_stdout();
});

// test to check that package can be reinstalled after manually removing the directory
forgetest!(can_reinstall_after_manual_remove, |prj, cmd| {
    cmd.git_init();

    let libs = prj.root().join("lib");
    let git_mod = prj.root().join(".git/modules/lib");
    let git_mod_file = prj.root().join(".gitmodules");

    let forge_std = libs.join("forge-std");
    let forge_std_mod = git_mod.join("forge-std");

    let install = |cmd: &mut TestCommand| {
        cmd.forge_fuse()
            .args(["install", "foundry-rs/forge-std", "--no-commit"])
            .assert_success()
            .stdout_eq(str![[r#"
Installing forge-std in [..] (url: Some("https://github.com/foundry-rs/forge-std"), tag: None)
    Installed forge-std[..]

"#]]);

        assert!(forge_std.exists());
        assert!(forge_std_mod.exists());

        let submods = read_string(&git_mod_file);
        assert!(submods.contains("https://github.com/foundry-rs/forge-std"));
    };

    install(&mut cmd);
    fs::remove_dir_all(forge_std.clone()).expect("Failed to remove forge-std");

    // install again
    install(&mut cmd);
});

// test that we can repeatedly install the same dependency without changes
forgetest!(can_install_repeatedly, |_prj, cmd| {
    cmd.git_init();

    cmd.forge_fuse().args(["install", "foundry-rs/forge-std"]);
    for _ in 0..3 {
        cmd.assert_success();
    }
});

// test that by default we install the latest semver release tag
// <https://github.com/openzeppelin/openzeppelin-contracts>
forgetest!(can_install_latest_release_tag, |prj, cmd| {
    cmd.git_init();
    cmd.forge_fuse().args(["install", "openzeppelin/openzeppelin-contracts"]);
    cmd.assert_success();

    let dep = prj.paths().libraries[0].join("openzeppelin-contracts");
    assert!(dep.exists());

    // the latest release at the time this test was written
    let version: Version = "4.8.0".parse().unwrap();
    let out = Command::new("git").current_dir(&dep).args(["describe", "--tags"]).output().unwrap();
    let tag = String::from_utf8_lossy(&out.stdout);
    let current: Version = tag.as_ref().trim_start_matches('v').trim().parse().unwrap();

    assert!(current >= version);
});

// Tests that forge update doesn't break a working dependency by recursively updating nested
// dependencies
forgetest!(
    #[cfg_attr(windows, ignore = "weird git fail")]
    can_update_library_with_outdated_nested_dependency,
    |prj, cmd| {
        cmd.git_init();

        let libs = prj.root().join("lib");
        let git_mod = prj.root().join(".git/modules/lib");
        let git_mod_file = prj.root().join(".gitmodules");

        // get paths to check inside install fn
        let package = libs.join("forge-5980-test");
        let package_mod = git_mod.join("forge-5980-test");

        // install main dependency
        cmd.forge_fuse()
            .args(["install", "evalir/forge-5980-test", "--no-commit"])
            .assert_success()
            .stdout_eq(str![[r#"
Installing forge-5980-test in [..] (url: Some("https://github.com/evalir/forge-5980-test"), tag: None)
    Installed forge-5980-test

"#]]);

        // assert paths exist
        assert!(package.exists());
        assert!(package_mod.exists());

        let submods = read_string(git_mod_file);
        assert!(submods.contains("https://github.com/evalir/forge-5980-test"));

        // try to update the top-level dependency; there should be no update for this dependency,
        // but its sub-dependency has upstream (breaking) changes; forge should not attempt to
        // update the sub-dependency
        cmd.forge_fuse().args(["update", "lib/forge-5980-test"]).assert_empty_stdout();

        // add explicit remappings for test file
        prj.update_config(|config| {
            config.remappings = vec![
                Remapping::from_str("forge-5980-test/=lib/forge-5980-test/src/").unwrap().into(),
                // explicit remapping for sub-dependendy seems necessary for some reason
                Remapping::from_str(
                    "forge-5980-test-dep/=lib/forge-5980-test/lib/forge-5980-test-dep/src/",
                )
                .unwrap()
                .into(),
            ];
        });

        // create test file that uses the top-level dependency; if the sub-dependency is updated,
        // compilation will fail
        prj.add_source(
            "CounterCopy",
            r#"
import "forge-5980-test/Counter.sol";
contract CounterCopy is Counter {
}
   "#,
        )
        .unwrap();

        // build and check output
        cmd.forge_fuse().arg("build").assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);
    }
);

const GAS_REPORT_CONTRACTS: &str = r#"
//SPDX-license-identifier: MIT

import "./test.sol";

contract ContractOne {
    int public i;

    constructor() {
        i = 0;
    }

    function foo() public{
        while(i<5){
            i++;
        }
    }
}

contract ContractOneTest is DSTest {
    ContractOne c1;

    function setUp() public {
        c1 = new ContractOne();
    }

    function testFoo() public {
        c1.foo();
    }
}


contract ContractTwo {
    int public i;

    constructor() {
        i = 0;
    }

    function bar() public{
        while(i<50){
            i++;
        }
    }
}

contract ContractTwoTest is DSTest {
    ContractTwo c2;

    function setUp() public {
        c2 = new ContractTwo();
    }

    function testBar() public {
        c2.bar();
    }
}

contract ContractThree {
    int public i;

    constructor() {
        i = 0;
    }

    function baz() public{
        while(i<500){
            i++;
        }
    }
}

contract ContractThreeTest is DSTest {
    ContractThree c3;

    function setUp() public {
        c3 = new ContractThree();
    }

    function testBaz() public {
        c3.baz();
    }
}
"#;

forgetest!(gas_report_all_contracts, |prj, cmd| {
    prj.insert_ds_test();
    prj.add_source("Contracts.sol", GAS_REPORT_CONTRACTS).unwrap();

    // report for all
    prj.update_config(|config| {
        config.gas_reports = vec!["*".to_string()];
        config.gas_reports_ignore = vec![];
    });

    cmd.forge_fuse().arg("test").arg("--gas-report").assert_success().stdout_eq(str![[r#"
...
╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractOne Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| foo                                    | 45656           | 45656 | 45656  | 45656 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯

╭------------------------------------------+-----------------+--------+--------+--------+---------╮
| src/Contracts.sol:ContractThree Contract |                 |        |        |        |         |
+=================================================================================================+
| Deployment Cost                          | Deployment Size |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| 133243                                   | 395             |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
|                                          |                 |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| Function Name                            | Min             | Avg    | Median | Max    | # Calls |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| baz                                      | 287711          | 287711 | 287711 | 287711 | 1       |
╰------------------------------------------+-----------------+--------+--------+--------+---------╯

╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractTwo Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| bar                                    | 67683           | 67683 | 67683  | 67683 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯


Ran 3 test suites [ELAPSED]: 3 tests passed, 0 failed, 0 skipped (3 total tests)

"#]]);
    cmd.forge_fuse().arg("test").arg("--gas-report").arg("--json").assert_success().stdout_eq(
        str![[r#"
[
  {
    "contract": "src/Contracts.sol:ContractOne",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "foo()": {
        "calls": 1,
        "min": 45656,
        "mean": 45656,
        "median": 45656,
        "max": 45656
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractThree",
    "deployment": {
      "gas": 133243,
      "size": 395
    },
    "functions": {
      "baz()": {
        "calls": 1,
        "min": 287711,
        "mean": 287711,
        "median": 287711,
        "max": 287711
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractTwo",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "bar()": {
        "calls": 1,
        "min": 67683,
        "mean": 67683,
        "median": 67683,
        "max": 67683
      }
    }
  }
]
"#]]
        .is_json(),
    );

    prj.update_config(|config| config.gas_reports = vec![]);
    cmd.forge_fuse().arg("test").arg("--gas-report").assert_success().stdout_eq(str![[r#"
...
╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractOne Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| foo                                    | 45656           | 45656 | 45656  | 45656 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯

╭------------------------------------------+-----------------+--------+--------+--------+---------╮
| src/Contracts.sol:ContractThree Contract |                 |        |        |        |         |
+=================================================================================================+
| Deployment Cost                          | Deployment Size |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| 133243                                   | 395             |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
|                                          |                 |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| Function Name                            | Min             | Avg    | Median | Max    | # Calls |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| baz                                      | 287711          | 287711 | 287711 | 287711 | 1       |
╰------------------------------------------+-----------------+--------+--------+--------+---------╯

╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractTwo Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| bar                                    | 67683           | 67683 | 67683  | 67683 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯


Ran 3 test suites [ELAPSED]: 3 tests passed, 0 failed, 0 skipped (3 total tests)

"#]]);
    cmd.forge_fuse().arg("test").arg("--gas-report").arg("--json").assert_success().stdout_eq(
        str![[r#"
[
  {
    "contract": "src/Contracts.sol:ContractOne",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "foo()": {
        "calls": 1,
        "min": 45656,
        "mean": 45656,
        "median": 45656,
        "max": 45656
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractThree",
    "deployment": {
      "gas": 133243,
      "size": 395
    },
    "functions": {
      "baz()": {
        "calls": 1,
        "min": 287711,
        "mean": 287711,
        "median": 287711,
        "max": 287711
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractTwo",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "bar()": {
        "calls": 1,
        "min": 67683,
        "mean": 67683,
        "median": 67683,
        "max": 67683
      }
    }
  }
]
"#]]
        .is_json(),
    );

    prj.update_config(|config| config.gas_reports = vec!["*".to_string()]);
    cmd.forge_fuse().arg("test").arg("--gas-report").assert_success().stdout_eq(str![[r#"
...
╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractOne Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| foo                                    | 45656           | 45656 | 45656  | 45656 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯

╭------------------------------------------+-----------------+--------+--------+--------+---------╮
| src/Contracts.sol:ContractThree Contract |                 |        |        |        |         |
+=================================================================================================+
| Deployment Cost                          | Deployment Size |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| 133243                                   | 395             |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
|                                          |                 |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| Function Name                            | Min             | Avg    | Median | Max    | # Calls |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| baz                                      | 287711          | 287711 | 287711 | 287711 | 1       |
╰------------------------------------------+-----------------+--------+--------+--------+---------╯

╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractTwo Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| bar                                    | 67683           | 67683 | 67683  | 67683 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯


Ran 3 test suites [ELAPSED]: 3 tests passed, 0 failed, 0 skipped (3 total tests)

"#]]);
    cmd.forge_fuse().arg("test").arg("--gas-report").arg("--json").assert_success().stdout_eq(
        str![[r#"
[
  {
    "contract": "src/Contracts.sol:ContractOne",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "foo()": {
        "calls": 1,
        "min": 45656,
        "mean": 45656,
        "median": 45656,
        "max": 45656
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractThree",
    "deployment": {
      "gas": 133243,
      "size": 395
    },
    "functions": {
      "baz()": {
        "calls": 1,
        "min": 287711,
        "mean": 287711,
        "median": 287711,
        "max": 287711
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractTwo",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "bar()": {
        "calls": 1,
        "min": 67683,
        "mean": 67683,
        "median": 67683,
        "max": 67683
      }
    }
  }
]
"#]]
        .is_json(),
    );

    prj.update_config(|config| {
        config.gas_reports =
            vec!["ContractOne".to_string(), "ContractTwo".to_string(), "ContractThree".to_string()];
    });
    cmd.forge_fuse().arg("test").arg("--gas-report").assert_success().stdout_eq(str![[r#"
...
╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractOne Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| foo                                    | 45656           | 45656 | 45656  | 45656 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯

╭------------------------------------------+-----------------+--------+--------+--------+---------╮
| src/Contracts.sol:ContractThree Contract |                 |        |        |        |         |
+=================================================================================================+
| Deployment Cost                          | Deployment Size |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| 133243                                   | 395             |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
|                                          |                 |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| Function Name                            | Min             | Avg    | Median | Max    | # Calls |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| baz                                      | 287711          | 287711 | 287711 | 287711 | 1       |
╰------------------------------------------+-----------------+--------+--------+--------+---------╯

╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractTwo Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| bar                                    | 67683           | 67683 | 67683  | 67683 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯


Ran 3 test suites [ELAPSED]: 3 tests passed, 0 failed, 0 skipped (3 total tests)

"#]]);
    cmd.forge_fuse().arg("test").arg("--gas-report").arg("--json").assert_success().stdout_eq(
        str![[r#"
[
  {
    "contract": "src/Contracts.sol:ContractOne",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "foo()": {
        "calls": 1,
        "min": 45656,
        "mean": 45656,
        "median": 45656,
        "max": 45656
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractThree",
    "deployment": {
      "gas": 133243,
      "size": 395
    },
    "functions": {
      "baz()": {
        "calls": 1,
        "min": 287711,
        "mean": 287711,
        "median": 287711,
        "max": 287711
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractTwo",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "bar()": {
        "calls": 1,
        "min": 67683,
        "mean": 67683,
        "median": 67683,
        "max": 67683
      }
    }
  }
]
"#]]
        .is_json(),
    );
});

forgetest!(gas_report_some_contracts, |prj, cmd| {
    prj.insert_ds_test();
    prj.add_source("Contracts.sol", GAS_REPORT_CONTRACTS).unwrap();

    // report for One
    prj.update_config(|config| config.gas_reports = vec!["ContractOne".to_string()]);
    cmd.forge_fuse();
    cmd.arg("test").arg("--gas-report").assert_success().stdout_eq(str![[r#"
...
╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractOne Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| foo                                    | 45656           | 45656 | 45656  | 45656 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯


Ran 3 test suites [ELAPSED]: 3 tests passed, 0 failed, 0 skipped (3 total tests)

"#]]);
    cmd.forge_fuse().arg("test").arg("--gas-report").arg("--json").assert_success().stdout_eq(
        str![[r#"
[
  {
    "contract": "src/Contracts.sol:ContractOne",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "foo()": {
        "calls": 1,
        "min": 45656,
        "mean": 45656,
        "median": 45656,
        "max": 45656
      }
    }
  }
]
"#]]
        .is_json(),
    );

    // report for Two
    prj.update_config(|config| config.gas_reports = vec!["ContractTwo".to_string()]);
    cmd.forge_fuse();
    cmd.arg("test").arg("--gas-report").assert_success().stdout_eq(str![[r#"
...
╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractTwo Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| bar                                    | 67683           | 67683 | 67683  | 67683 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯


Ran 3 test suites [ELAPSED]: 3 tests passed, 0 failed, 0 skipped (3 total tests)

"#]]);
    cmd.forge_fuse().arg("test").arg("--gas-report").arg("--json").assert_success().stdout_eq(
        str![[r#"
[
  {
    "contract": "src/Contracts.sol:ContractTwo",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "bar()": {
        "calls": 1,
        "min": 67683,
        "mean": 67683,
        "median": 67683,
        "max": 67683
      }
    }
  }
]
"#]]
        .is_json(),
    );

    // report for Three
    prj.update_config(|config| config.gas_reports = vec!["ContractThree".to_string()]);
    cmd.forge_fuse();
    cmd.arg("test").arg("--gas-report").assert_success().stdout_eq(str![[r#"
...
╭------------------------------------------+-----------------+--------+--------+--------+---------╮
| src/Contracts.sol:ContractThree Contract |                 |        |        |        |         |
+=================================================================================================+
| Deployment Cost                          | Deployment Size |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| 133243                                   | 395             |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
|                                          |                 |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| Function Name                            | Min             | Avg    | Median | Max    | # Calls |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| baz                                      | 287711          | 287711 | 287711 | 287711 | 1       |
╰------------------------------------------+-----------------+--------+--------+--------+---------╯


Ran 3 test suites [ELAPSED]: 3 tests passed, 0 failed, 0 skipped (3 total tests)

"#]]);
    cmd.forge_fuse().arg("test").arg("--gas-report").arg("--json").assert_success().stdout_eq(
        str![[r#"
[
  {
    "contract": "src/Contracts.sol:ContractThree",
    "deployment": {
      "gas": 133243,
      "size": 395
    },
    "functions": {
      "baz()": {
        "calls": 1,
        "min": 287711,
        "mean": 287711,
        "median": 287711,
        "max": 287711
      }
    }
  }
]
"#]]
        .is_json(),
    );
});

forgetest!(gas_report_ignore_some_contracts, |prj, cmd| {
    prj.insert_ds_test();
    prj.add_source("Contracts.sol", GAS_REPORT_CONTRACTS).unwrap();

    // ignore ContractOne
    prj.update_config(|config| {
        config.gas_reports = vec!["*".to_string()];
        config.gas_reports_ignore = vec!["ContractOne".to_string()];
    });
    cmd.forge_fuse();
    cmd.arg("test").arg("--gas-report").assert_success().stdout_eq(str![[r#"
...
╭------------------------------------------+-----------------+--------+--------+--------+---------╮
| src/Contracts.sol:ContractThree Contract |                 |        |        |        |         |
+=================================================================================================+
| Deployment Cost                          | Deployment Size |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| 133243                                   | 395             |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
|                                          |                 |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| Function Name                            | Min             | Avg    | Median | Max    | # Calls |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| baz                                      | 287711          | 287711 | 287711 | 287711 | 1       |
╰------------------------------------------+-----------------+--------+--------+--------+---------╯

╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractTwo Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| bar                                    | 67683           | 67683 | 67683  | 67683 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯


Ran 3 test suites [ELAPSED]: 3 tests passed, 0 failed, 0 skipped (3 total tests)

"#]]);
    cmd.forge_fuse().arg("test").arg("--gas-report").arg("--json").assert_success().stdout_eq(
        str![[r#"
[
  {
    "contract": "src/Contracts.sol:ContractThree",
    "deployment": {
      "gas": 133243,
      "size": 395
    },
    "functions": {
      "baz()": {
        "calls": 1,
        "min": 287711,
        "mean": 287711,
        "median": 287711,
        "max": 287711
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractTwo",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "bar()": {
        "calls": 1,
        "min": 67683,
        "mean": 67683,
        "median": 67683,
        "max": 67683
      }
    }
  }
]
"#]]
        .is_json(),
    );

    // ignore ContractTwo
    cmd.forge_fuse();
    prj.update_config(|config| {
        config.gas_reports = vec![];
        config.gas_reports_ignore = vec!["ContractTwo".to_string()];
    });
    cmd.forge_fuse();
    cmd.arg("test").arg("--gas-report").assert_success().stdout_eq(str![[r#"
...
╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractOne Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| foo                                    | 45656           | 45656 | 45656  | 45656 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯

╭------------------------------------------+-----------------+--------+--------+--------+---------╮
| src/Contracts.sol:ContractThree Contract |                 |        |        |        |         |
+=================================================================================================+
| Deployment Cost                          | Deployment Size |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| 133243                                   | 395             |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
|                                          |                 |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| Function Name                            | Min             | Avg    | Median | Max    | # Calls |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| baz                                      | 287711          | 287711 | 287711 | 287711 | 1       |
╰------------------------------------------+-----------------+--------+--------+--------+---------╯


Ran 3 test suites [ELAPSED]: 3 tests passed, 0 failed, 0 skipped (3 total tests)

"#]]);
    cmd.forge_fuse().arg("test").arg("--gas-report").arg("--json").assert_success().stdout_eq(
        str![[r#"
[
  {
    "contract": "src/Contracts.sol:ContractOne",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "foo()": {
        "calls": 1,
        "min": 45656,
        "mean": 45656,
        "median": 45656,
        "max": 45656
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractThree",
    "deployment": {
      "gas": 133243,
      "size": 395
    },
    "functions": {
      "baz()": {
        "calls": 1,
        "min": 287711,
        "mean": 287711,
        "median": 287711,
        "max": 287711
      }
    }
  }
]
"#]]
        .is_json(),
    );

    // If the user listed the contract in 'gas_reports' (the foundry.toml field) a
    // report for the contract is generated even if it's listed in the ignore
    // list. This is addressed this way because getting a report you don't expect is
    // preferable than not getting one you expect. A warning is printed to stderr
    // indicating the "double listing".
    cmd.forge_fuse();
    prj.update_config(|config| {
        config.gas_reports =
            vec!["ContractOne".to_string(), "ContractTwo".to_string(), "ContractThree".to_string()];
        config.gas_reports_ignore = vec!["ContractThree".to_string()];
    });
    cmd.forge_fuse();
    cmd.arg("test")
        .arg("--gas-report")
        .assert_success()
        .stdout_eq(str![[r#"
...
╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractOne Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| foo                                    | 45656           | 45656 | 45656  | 45656 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯

╭------------------------------------------+-----------------+--------+--------+--------+---------╮
| src/Contracts.sol:ContractThree Contract |                 |        |        |        |         |
+=================================================================================================+
| Deployment Cost                          | Deployment Size |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| 133243                                   | 395             |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
|                                          |                 |        |        |        |         |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| Function Name                            | Min             | Avg    | Median | Max    | # Calls |
|------------------------------------------+-----------------+--------+--------+--------+---------|
| baz                                      | 287711          | 287711 | 287711 | 287711 | 1       |
╰------------------------------------------+-----------------+--------+--------+--------+---------╯

╭----------------------------------------+-----------------+-------+--------+-------+---------╮
| src/Contracts.sol:ContractTwo Contract |                 |       |        |       |         |
+=============================================================================================+
| Deployment Cost                        | Deployment Size |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| 133027                                 | 394             |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
|                                        |                 |       |        |       |         |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                          | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------------+-----------------+-------+--------+-------+---------|
| bar                                    | 67683           | 67683 | 67683  | 67683 | 1       |
╰----------------------------------------+-----------------+-------+--------+-------+---------╯


Ran 3 test suites [ELAPSED]: 3 tests passed, 0 failed, 0 skipped (3 total tests)

"#]])
        .stderr_eq(str![[r#"
...
Warning: ContractThree is listed in both 'gas_reports' and 'gas_reports_ignore'.
...
"#]]);
    cmd.forge_fuse()
        .arg("test")
        .arg("--gas-report")
        .arg("--json")
        .assert_success()
        .stdout_eq(
            str![[r#"
[
  {
    "contract": "src/Contracts.sol:ContractOne",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "foo()": {
        "calls": 1,
        "min": 45656,
        "mean": 45656,
        "median": 45656,
        "max": 45656
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractThree",
    "deployment": {
      "gas": 133243,
      "size": 395
    },
    "functions": {
      "baz()": {
        "calls": 1,
        "min": 287711,
        "mean": 287711,
        "median": 287711,
        "max": 287711
      }
    }
  },
  {
    "contract": "src/Contracts.sol:ContractTwo",
    "deployment": {
      "gas": 133027,
      "size": 394
    },
    "functions": {
      "bar()": {
        "calls": 1,
        "min": 67683,
        "mean": 67683,
        "median": 67683,
        "max": 67683
      }
    }
  }
]
"#]]
            .is_json(),
        )
        .stderr_eq(str![[r#"
...
Warning: ContractThree is listed in both 'gas_reports' and 'gas_reports_ignore'.
...
"#]]);
});

forgetest!(gas_report_flatten_multiple_selectors, |prj, cmd| {
    prj.insert_ds_test();
    prj.add_source(
        "Counter.sol",
        r#"
contract Counter {
    uint256 public a;
    int256 public b;

    function setNumber(uint256 x) public {
        a = x;
    }

    function setNumber(int256 x) public {
        b = x;
    }
}
"#,
    )
    .unwrap();

    prj.add_source(
        "CounterTest.t.sol",
        r#"
import "./test.sol";
import {Counter} from "./Counter.sol";

contract CounterTest is DSTest {
    Counter public counter;

    function setUp() public {
        counter = new Counter();
        counter.setNumber(uint256(0));
        counter.setNumber(int256(0));
    }

    function test_Increment() public {
        counter.setNumber(uint256(counter.a() + 1));
        counter.setNumber(int256(counter.b() + 1));
    }
}
"#,
    )
    .unwrap();

    cmd.arg("test").arg("--gas-report").assert_success().stdout_eq(str![[r#"
...
╭----------------------------------+-----------------+-------+--------+-------+---------╮
| src/Counter.sol:Counter Contract |                 |       |        |       |         |
+=======================================================================================+
| Deployment Cost                  | Deployment Size |       |        |       |         |
|----------------------------------+-----------------+-------+--------+-------+---------|
| 172107                           | 578             |       |        |       |         |
|----------------------------------+-----------------+-------+--------+-------+---------|
|                                  |                 |       |        |       |         |
|----------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                    | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------+-----------------+-------+--------+-------+---------|
| a                                | 2402            | 2402  | 2402   | 2402  | 1       |
|----------------------------------+-----------------+-------+--------+-------+---------|
| b                                | 2447            | 2447  | 2447   | 2447  | 1       |
|----------------------------------+-----------------+-------+--------+-------+---------|
| setNumber(int256)                | 23851           | 33807 | 33807  | 43763 | 2       |
|----------------------------------+-----------------+-------+--------+-------+---------|
| setNumber(uint256)               | 23806           | 33762 | 33762  | 43718 | 2       |
╰----------------------------------+-----------------+-------+--------+-------+---------╯


Ran 1 test suite [ELAPSED]: 1 tests passed, 0 failed, 0 skipped (1 total tests)

"#]]);
    cmd.forge_fuse().arg("test").arg("--gas-report").arg("--json").assert_success().stdout_eq(
        str![[r#"
[
  {
    "contract": "src/Counter.sol:Counter",
    "deployment": {
      "gas": 172107,
      "size": 578
    },
    "functions": {
      "a()": {
        "calls": 1,
        "min": 2402,
        "mean": 2402,
        "median": 2402,
        "max": 2402
      },
      "b()": {
        "calls": 1,
        "min": 2447,
        "mean": 2447,
        "median": 2447,
        "max": 2447
      },
      "setNumber(int256)": {
        "calls": 2,
        "min": 23851,
        "mean": 33807,
        "median": 33807,
        "max": 43763
      },
      "setNumber(uint256)": {
        "calls": 2,
        "min": 23806,
        "mean": 33762,
        "median": 33762,
        "max": 43718
      }
    }
  }
]
"#]]
        .is_json(),
    );
});

// <https://github.com/foundry-rs/foundry/issues/9115>
forgetest_init!(gas_report_with_fallback, |prj, cmd| {
    prj.add_test(
        "DelegateProxyTest.sol",
        r#"
import {Test} from "forge-std/Test.sol";

contract ProxiedContract {
    uint256 public amount;

    function deposit(uint256 aba) external {
        amount = amount * 2;
    }

    function deposit() external {
    }
}

contract DelegateProxy {
    address internal implementation;

    constructor(address counter) {
        implementation = counter;
    }

    function deposit() external {
    }

    fallback() external payable {
        address addr = implementation;

        assembly {
            calldatacopy(0, 0, calldatasize())
            let result := delegatecall(gas(), addr, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch result
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }
}

contract GasReportFallbackTest is Test {
    function test_fallback_gas_report() public {
        ProxiedContract proxied = ProxiedContract(address(new DelegateProxy(address(new ProxiedContract()))));
        proxied.deposit(100);
        proxied.deposit();
    }
}
"#,
    )
    .unwrap();

    cmd.args(["test", "--mt", "test_fallback_gas_report", "-vvvv", "--gas-report"])
        .assert_success()
        .stdout_eq(str![[r#"
...
╭---------------------------------------------------+-----------------+-------+--------+-------+---------╮
| test/DelegateProxyTest.sol:DelegateProxy Contract |                 |       |        |       |         |
+========================================================================================================+
| Deployment Cost                                   | Deployment Size |       |        |       |         |
|---------------------------------------------------+-----------------+-------+--------+-------+---------|
| 117171                                            | 471             |       |        |       |         |
|---------------------------------------------------+-----------------+-------+--------+-------+---------|
|                                                   |                 |       |        |       |         |
|---------------------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                                     | Min             | Avg   | Median | Max   | # Calls |
|---------------------------------------------------+-----------------+-------+--------+-------+---------|
| deposit                                           | 21185           | 21185 | 21185  | 21185 | 1       |
|---------------------------------------------------+-----------------+-------+--------+-------+---------|
| fallback                                          | 29758           | 29758 | 29758  | 29758 | 1       |
╰---------------------------------------------------+-----------------+-------+--------+-------+---------╯

╭-----------------------------------------------------+-----------------+------+--------+------+---------╮
| test/DelegateProxyTest.sol:ProxiedContract Contract |                 |      |        |      |         |
+========================================================================================================+
| Deployment Cost                                     | Deployment Size |      |        |      |         |
|-----------------------------------------------------+-----------------+------+--------+------+---------|
| 153531                                              | 494             |      |        |      |         |
|-----------------------------------------------------+-----------------+------+--------+------+---------|
|                                                     |                 |      |        |      |         |
|-----------------------------------------------------+-----------------+------+--------+------+---------|
| Function Name                                       | Min             | Avg  | Median | Max  | # Calls |
|-----------------------------------------------------+-----------------+------+--------+------+---------|
| deposit                                             | 3661            | 3661 | 3661   | 3661 | 1       |
╰-----------------------------------------------------+-----------------+------+--------+------+---------╯


Ran 1 test suite [ELAPSED]: 1 tests passed, 0 failed, 0 skipped (1 total tests)

"#]]);

    cmd.forge_fuse()
        .args(["test", "--mt", "test_fallback_gas_report", "--gas-report", "--json"])
        .assert_success()
        .stdout_eq(
            str![[r#"
[
  {
    "contract": "test/DelegateProxyTest.sol:DelegateProxy",
    "deployment": {
      "gas": 117171,
      "size": 471
    },
    "functions": {
      "deposit()": {
        "calls": 1,
        "min": 21185,
        "mean": 21185,
        "median": 21185,
        "max": 21185
      },
      "fallback()": {
        "calls": 1,
        "min": 29758,
        "mean": 29758,
        "median": 29758,
        "max": 29758
      }
    }
  },
  {
    "contract": "test/DelegateProxyTest.sol:ProxiedContract",
    "deployment": {
      "gas": 153531,
      "size": 494
    },
    "functions": {
      "deposit(uint256)": {
        "calls": 1,
        "min": 3661,
        "mean": 3661,
        "median": 3661,
        "max": 3661
      }
    }
  }
]
"#]]
            .is_json(),
        );
});

// <https://github.com/foundry-rs/foundry/issues/9300>
forgetest_init!(gas_report_size_for_nested_create, |prj, cmd| {
    prj.add_test(
        "NestedDeployTest.sol",
        r#"
import {Test} from "forge-std/Test.sol";
contract Child {
    AnotherChild public child;
    constructor() {
        child = new AnotherChild();
    }
    function w() external {
        child.w();
    }
}
contract AnotherChild {
    function w() external {}
}
contract Parent {
    Child public immutable child;
    constructor() {
        child = new Child();
    }
}
contract NestedDeploy is Test {
    function test_nested_create_gas_report() external {
        Parent p = new Parent();
        p.child().child().w();
    }
}
"#,
    )
    .unwrap();

    cmd.args(["test", "--mt", "test_nested_create_gas_report", "--gas-report"])
        .assert_success()
        .stdout_eq(str![[r#"
...
╭-------------------------------------------------+-----------------+-------+--------+-------+---------╮
| test/NestedDeployTest.sol:AnotherChild Contract |                 |       |        |       |         |
+======================================================================================================+
| Deployment Cost                                 | Deployment Size |       |        |       |         |
|-------------------------------------------------+-----------------+-------+--------+-------+---------|
| 0                                               | 132             |       |        |       |         |
|-------------------------------------------------+-----------------+-------+--------+-------+---------|
|                                                 |                 |       |        |       |         |
|-------------------------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                                   | Min             | Avg   | Median | Max   | # Calls |
|-------------------------------------------------+-----------------+-------+--------+-------+---------|
| w                                               | 21185           | 21185 | 21185  | 21185 | 1       |
╰-------------------------------------------------+-----------------+-------+--------+-------+---------╯

╭------------------------------------------+-----------------+-----+--------+-----+---------╮
| test/NestedDeployTest.sol:Child Contract |                 |     |        |     |         |
+===========================================================================================+
| Deployment Cost                          | Deployment Size |     |        |     |         |
|------------------------------------------+-----------------+-----+--------+-----+---------|
| 0                                        | 731             |     |        |     |         |
|------------------------------------------+-----------------+-----+--------+-----+---------|
|                                          |                 |     |        |     |         |
|------------------------------------------+-----------------+-----+--------+-----+---------|
| Function Name                            | Min             | Avg | Median | Max | # Calls |
|------------------------------------------+-----------------+-----+--------+-----+---------|
| child                                    | 681             | 681 | 681    | 681 | 1       |
╰------------------------------------------+-----------------+-----+--------+-----+---------╯

╭-------------------------------------------+-----------------+-----+--------+-----+---------╮
| test/NestedDeployTest.sol:Parent Contract |                 |     |        |     |         |
+============================================================================================+
| Deployment Cost                           | Deployment Size |     |        |     |         |
|-------------------------------------------+-----------------+-----+--------+-----+---------|
| 328961                                    | 1163            |     |        |     |         |
|-------------------------------------------+-----------------+-----+--------+-----+---------|
|                                           |                 |     |        |     |         |
|-------------------------------------------+-----------------+-----+--------+-----+---------|
| Function Name                             | Min             | Avg | Median | Max | # Calls |
|-------------------------------------------+-----------------+-----+--------+-----+---------|
| child                                     | 525             | 525 | 525    | 525 | 1       |
╰-------------------------------------------+-----------------+-----+--------+-----+---------╯


Ran 1 test suite [ELAPSED]: 1 tests passed, 0 failed, 0 skipped (1 total tests)

"#]]);

    cmd.forge_fuse()
        .args(["test", "--mt", "test_nested_create_gas_report", "--gas-report", "--json"])
        .assert_success()
        .stdout_eq(
            str![[r#"
[
  {
    "contract": "test/NestedDeployTest.sol:AnotherChild",
    "deployment": {
      "gas": 0,
      "size": 132
    },
    "functions": {
      "w()": {
        "calls": 1,
        "min": 21185,
        "mean": 21185,
        "median": 21185,
        "max": 21185
      }
    }
  },
  {
    "contract": "test/NestedDeployTest.sol:Child",
    "deployment": {
      "gas": 0,
      "size": 731
    },
    "functions": {
      "child()": {
        "calls": 1,
        "min": 681,
        "mean": 681,
        "median": 681,
        "max": 681
      }
    }
  },
  {
    "contract": "test/NestedDeployTest.sol:Parent",
    "deployment": {
      "gas": 328961,
      "size": 1163
    },
    "functions": {
      "child()": {
        "calls": 1,
        "min": 525,
        "mean": 525,
        "median": 525,
        "max": 525
      }
    }
  }
]
"#]]
            .is_json(),
        );
});

forgetest_init!(can_use_absolute_imports, |prj, cmd| {
    prj.update_config(|config| {
        let remapping = prj.paths().libraries[0].join("myDependency");
        config.remappings =
            vec![Remapping::from_str(&format!("myDependency/={}", remapping.display()))
                .unwrap()
                .into()];
    });

    prj.add_lib(
        "myDependency/src/interfaces/IConfig.sol",
        r"

    interface IConfig {}
   ",
    )
    .unwrap();

    prj.add_lib(
        "myDependency/src/Config.sol",
        r#"
        import "src/interfaces/IConfig.sol";

    contract Config {}
   "#,
    )
    .unwrap();

    prj.add_source(
        "Greeter",
        r#"
        import "myDependency/src/Config.sol";

    contract Greeter {}
   "#,
    )
    .unwrap();

    cmd.arg("build").assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);
});

// <https://github.com/foundry-rs/foundry/issues/3440>
forgetest_init!(can_use_absolute_imports_from_test_and_script, |prj, cmd| {
    prj.add_script(
        "IMyScript.sol",
        r"
interface IMyScript {}
        ",
    )
    .unwrap();

    prj.add_script(
        "MyScript.sol",
        r#"
import "script/IMyScript.sol";

contract MyScript is IMyScript {}
        "#,
    )
    .unwrap();

    prj.add_test(
        "IMyTest.sol",
        r"
interface IMyTest {}
        ",
    )
    .unwrap();

    prj.add_test(
        "MyTest.sol",
        r#"
import "test/IMyTest.sol";

contract MyTest is IMyTest {}
    "#,
    )
    .unwrap();

    cmd.arg("build").assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);
});

// checks `forge inspect <contract> irOptimized works and does not strip comments
forgetest_init!(can_inspect_ir_optimized, |_prj, cmd| {
    cmd.args(["inspect", TEMPLATE_CONTRACT, "irOptimized"]);
    cmd.assert_success().stdout_eq(str![[r#"
/// @use-src 0:"src/Counter.sol"
object "Counter_21" {
    code {
        {
            /// @src 0:65:257  "contract Counter {..."
...
"#]]);
});

// checks forge bind works correctly on the default project
forgetest_init!(can_bind, |prj, cmd| {
    prj.clear();

    cmd.arg("bind").assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!
Generating bindings for [..] contracts
Bindings have been generated to [..]

"#]]);
});

// checks missing dependencies are auto installed
forgetest_init!(can_install_missing_deps_test, |prj, cmd| {
    prj.clear();

    // wipe forge-std
    let forge_std_dir = prj.root().join("lib/forge-std");
    pretty_err(&forge_std_dir, fs::remove_dir_all(&forge_std_dir));

    cmd.arg("test").assert_success().stdout_eq(str![[r#"
Missing dependencies found. Installing now...

[UPDATING_DEPENDENCIES]
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

Ran 2 tests for test/Counter.t.sol:CounterTest
[PASS] testFuzz_SetNumber(uint256) (runs: 256, [AVG_GAS])
[PASS] test_Increment() ([GAS])
Suite result: ok. 2 passed; 0 failed; 0 skipped; [ELAPSED]

Ran 1 test suite [ELAPSED]: 2 tests passed, 0 failed, 0 skipped (2 total tests)

"#]]);
});

// checks missing dependencies are auto installed
forgetest_init!(can_install_missing_deps_build, |prj, cmd| {
    prj.clear();

    // wipe forge-std
    let forge_std_dir = prj.root().join("lib/forge-std");
    pretty_err(&forge_std_dir, fs::remove_dir_all(&forge_std_dir));

    // Build the project
    cmd.arg("build").assert_success().stdout_eq(str![[r#"
Missing dependencies found. Installing now...

[UPDATING_DEPENDENCIES]
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    // Expect compilation to be skipped as no files have changed
    cmd.forge_fuse().arg("build").assert_success().stdout_eq(str![[r#"
No files changed, compilation skipped

"#]]);
});

// checks that extra output works
forgetest_init!(can_build_skip_contracts, |prj, cmd| {
    prj.clear();

    // Only builds the single template contract `src/*`
    cmd.args(["build", "--skip", "tests", "--skip", "scripts"]).assert_success().stdout_eq(str![[
        r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#
    ]]);

    // Expect compilation to be skipped as no files have changed
    cmd.arg("build").assert_success().stdout_eq(str![[r#"
No files changed, compilation skipped

"#]]);
});

forgetest_init!(can_build_skip_glob, |prj, cmd| {
    prj.add_test(
        "Foo",
        r"
contract TestDemo {
function test_run() external {}
}",
    )
    .unwrap();

    // only builds the single template contract `src/*` even if `*.t.sol` or `.s.sol` is absent
    prj.clear();
    cmd.args(["build", "--skip", "*/test/**", "--skip", "*/script/**", "--force"])
        .assert_success()
        .stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    cmd.forge_fuse()
        .args(["build", "--skip", "./test/**", "--skip", "./script/**", "--force"])
        .assert_success()
        .stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);
});

forgetest_init!(can_build_specific_paths, |prj, cmd| {
    prj.wipe();
    prj.add_source(
        "Counter.sol",
        r"
contract Counter {
function count() external {}
}",
    )
    .unwrap();
    prj.add_test(
        "Foo.sol",
        r"
contract Foo {
function test_foo() external {}
}",
    )
    .unwrap();
    prj.add_test(
        "Bar.sol",
        r"
contract Bar {
function test_bar() external {}
}",
    )
    .unwrap();

    // Build 2 files within test dir
    prj.clear();
    cmd.args(["build", "test", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    // Build one file within src dir
    prj.clear();
    cmd.forge_fuse();
    cmd.args(["build", "src", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    // Build 3 files from test and src dirs
    prj.clear();
    cmd.forge_fuse();
    cmd.args(["build", "src", "test", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    // Build single test file
    prj.clear();
    cmd.forge_fuse();
    cmd.args(["build", "test/Bar.sol", "--force"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!

"#]]);

    // Fail if no source file found.
    prj.clear();
    cmd.forge_fuse();
    cmd.args(["build", "test/Dummy.sol", "--force"]).assert_failure().stderr_eq(str![[r#"
Error: No source files found in specified build paths.

"#]]);
});

// checks that build --sizes includes all contracts even if unchanged
forgetest_init!(can_build_sizes_repeatedly, |prj, cmd| {
    prj.clear_cache();

    cmd.args(["build", "--sizes"]).assert_success().stdout_eq(str![[r#"
...
╭----------+------------------+-------------------+--------------------+---------------------╮
| Contract | Runtime Size (B) | Initcode Size (B) | Runtime Margin (B) | Initcode Margin (B) |
+============================================================================================+
| Counter  | 481              | 509               | 24,095             | 48,643              |
╰----------+------------------+-------------------+--------------------+---------------------╯


"#]]);

    cmd.forge_fuse().args(["build", "--sizes", "--json"]).assert_success().stdout_eq(
        str![[r#"
{
  "Counter": {
    "runtime_size": 481,
    "init_size": 509,
    "runtime_margin": 24095,
    "init_margin": 48643
  }
}
"#]]
        .is_json(),
    );
});

// checks that build --names includes all contracts even if unchanged
forgetest_init!(can_build_names_repeatedly, |prj, cmd| {
    prj.clear_cache();

    cmd.args(["build", "--names"]).assert_success().stdout_eq(str![[r#"
[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!
  compiler version: [..]
    - [..]
...

"#]]);

    cmd.forge_fuse()
        .args(["build", "--names", "--json"])
        .assert_success()
        .stdout_eq(str![[r#""{...}""#]].is_json());
});

forgetest_init!(can_inspect_counter_pretty, |prj, cmd| {
    cmd.args(["inspect", "src/Counter.sol:Counter", "abi"]).assert_success().stdout_eq(str![[r#"

╭----------+---------------------------------+------------╮
| Type     | Signature                       | Selector   |
+=========================================================+
| function | increment() nonpayable          | 0xd09de08a |
|----------+---------------------------------+------------|
| function | number() view returns (uint256) | 0x8381f58a |
|----------+---------------------------------+------------|
| function | setNumber(uint256) nonpayable   | 0x3fb5c1cb |
╰----------+---------------------------------+------------╯


"#]]);
});

const CUSTOM_COUNTER: &str = r#"
    contract Counter {
    uint256 public number;
    uint64 public count;
    struct MyStruct {
        uint64 count;
    }
    struct ErrWithMsg {
        string message;
    }

    event Incremented(uint256 newValue);
    event Decremented(uint256 newValue);

    error NumberIsZero();
    error CustomErr(ErrWithMsg e);

    constructor(uint256 _number) {
        number = _number;
    }

    function setNumber(uint256 newNumber) public {
        number = newNumber;
    }

    function increment() external {
        number++;
    }

    function decrement() public payable {
        if (number == 0) {
            return;
        }
        number--;
    }

    function square() public {
        number = number * number;
    }

    fallback() external payable {
        ErrWithMsg memory err = ErrWithMsg("Fallback function is not allowed");
        revert CustomErr(err);
    }

    receive() external payable {
        count++;
    }

    function setStruct(MyStruct memory s, uint32 b) public {
        count = s.count;
    }
}
    "#;
forgetest!(inspect_custom_counter_abi, |prj, cmd| {
    prj.add_source("Counter.sol", CUSTOM_COUNTER).unwrap();

    cmd.args(["inspect", "Counter", "abi"]).assert_success().stdout_eq(str![[r#"

╭-------------+-----------------------------------------------+--------------------------------------------------------------------╮
| Type        | Signature                                     | Selector                                                           |
+==================================================================================================================================+
| event       | Decremented(uint256)                          | 0xc9118d86370931e39644ee137c931308fa3774f6c90ab057f0c3febf427ef94a |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| event       | Incremented(uint256)                          | 0x20d8a6f5a693f9d1d627a598e8820f7a55ee74c183aa8f1a30e8d4e8dd9a8d84 |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| error       | CustomErr(Counter.ErrWithMsg)                 | 0x0625625a                                                         |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| error       | NumberIsZero()                                | 0xde5d32ac                                                         |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| function    | count() view returns (uint64)                 | 0x06661abd                                                         |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| function    | decrement() payable                           | 0x2baeceb7                                                         |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| function    | increment() nonpayable                        | 0xd09de08a                                                         |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| function    | number() view returns (uint256)               | 0x8381f58a                                                         |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| function    | setNumber(uint256) nonpayable                 | 0x3fb5c1cb                                                         |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| function    | setStruct(Counter.MyStruct,uint32) nonpayable | 0x08ef7366                                                         |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| function    | square() nonpayable                           | 0xd742cb01                                                         |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| constructor | constructor(uint256) nonpayable               |                                                                    |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| fallback    | fallback() payable                            |                                                                    |
|-------------+-----------------------------------------------+--------------------------------------------------------------------|
| receive     | receive() payable                             |                                                                    |
╰-------------+-----------------------------------------------+--------------------------------------------------------------------╯


"#]]);
});

forgetest!(inspect_custom_counter_events, |prj, cmd| {
    prj.add_source("Counter.sol", CUSTOM_COUNTER).unwrap();

    cmd.args(["inspect", "Counter", "events"]).assert_success().stdout_eq(str![[r#"

╭----------------------+--------------------------------------------------------------------╮
| Event                | Topic                                                              |
+===========================================================================================+
| Decremented(uint256) | 0xc9118d86370931e39644ee137c931308fa3774f6c90ab057f0c3febf427ef94a |
|----------------------+--------------------------------------------------------------------|
| Incremented(uint256) | 0x20d8a6f5a693f9d1d627a598e8820f7a55ee74c183aa8f1a30e8d4e8dd9a8d84 |
╰----------------------+--------------------------------------------------------------------╯


"#]]);
});

forgetest!(inspect_custom_counter_errors, |prj, cmd| {
    prj.add_source("Counter.sol", CUSTOM_COUNTER).unwrap();

    cmd.args(["inspect", "Counter", "errors"]).assert_success().stdout_eq(str![[r#"

╭-------------------------------+----------╮
| Error                         | Selector |
+==========================================+
| CustomErr(Counter.ErrWithMsg) | 0625625a |
|-------------------------------+----------|
| NumberIsZero()                | de5d32ac |
╰-------------------------------+----------╯


"#]]);
});

forgetest!(inspect_custom_counter_method_identifiers, |prj, cmd| {
    prj.add_source("Counter.sol", CUSTOM_COUNTER).unwrap();

    cmd.args(["inspect", "Counter", "method-identifiers"]).assert_success().stdout_eq(str![[r#"

╭----------------------------+------------╮
| Method                     | Identifier |
+=========================================+
| count()                    | 06661abd   |
|----------------------------+------------|
| decrement()                | 2baeceb7   |
|----------------------------+------------|
| increment()                | d09de08a   |
|----------------------------+------------|
| number()                   | 8381f58a   |
|----------------------------+------------|
| setNumber(uint256)         | 3fb5c1cb   |
|----------------------------+------------|
| setStruct((uint64),uint32) | 08ef7366   |
|----------------------------+------------|
| square()                   | d742cb01   |
╰----------------------------+------------╯


"#]]);
});

// checks that `clean` also works with the "out" value set in Config
forgetest_init!(gas_report_include_tests, |prj, cmd| {
    prj.update_config(|config| {
        config.gas_reports_include_tests = true;
        config.fuzz.runs = 1;
    });

    cmd.args(["test", "--match-test", "test_Increment", "--gas-report"])
        .assert_success()
        .stdout_eq(str![[r#"
...
╭----------------------------------+-----------------+-------+--------+-------+---------╮
| src/Counter.sol:Counter Contract |                 |       |        |       |         |
+=======================================================================================+
| Deployment Cost                  | Deployment Size |       |        |       |         |
|----------------------------------+-----------------+-------+--------+-------+---------|
| 156813                           | 509             |       |        |       |         |
|----------------------------------+-----------------+-------+--------+-------+---------|
|                                  |                 |       |        |       |         |
|----------------------------------+-----------------+-------+--------+-------+---------|
| Function Name                    | Min             | Avg   | Median | Max   | # Calls |
|----------------------------------+-----------------+-------+--------+-------+---------|
| increment                        | 43482           | 43482 | 43482  | 43482 | 1       |
|----------------------------------+-----------------+-------+--------+-------+---------|
| number                           | 424             | 424   | 424    | 424   | 1       |
|----------------------------------+-----------------+-------+--------+-------+---------|
| setNumber                        | 23784           | 23784 | 23784  | 23784 | 1       |
╰----------------------------------+-----------------+-------+--------+-------+---------╯

╭-----------------------------------------+-----------------+--------+--------+--------+---------╮
| test/Counter.t.sol:CounterTest Contract |                 |        |        |        |         |
+================================================================================================+
| Deployment Cost                         | Deployment Size |        |        |        |         |
|-----------------------------------------+-----------------+--------+--------+--------+---------|
| 1545498                                 | 7578            |        |        |        |         |
|-----------------------------------------+-----------------+--------+--------+--------+---------|
|                                         |                 |        |        |        |         |
|-----------------------------------------+-----------------+--------+--------+--------+---------|
| Function Name                           | Min             | Avg    | Median | Max    | # Calls |
|-----------------------------------------+-----------------+--------+--------+--------+---------|
| setUp                                   | 218902          | 218902 | 218902 | 218902 | 1       |
|-----------------------------------------+-----------------+--------+--------+--------+---------|
| test_Increment                          | 52915           | 52915  | 52915  | 52915  | 1       |
╰-----------------------------------------+-----------------+--------+--------+--------+---------╯


Ran 1 test suite [ELAPSED]: 1 tests passed, 0 failed, 0 skipped (1 total tests)

"#]]);

    cmd.forge_fuse()
        .args(["test", "--mt", "test_Increment", "--gas-report", "--json"])
        .assert_success()
        .stdout_eq(
            str![[r#"
[
  {
    "contract": "src/Counter.sol:Counter",
    "deployment": {
      "gas": 156813,
      "size": 509
    },
    "functions": {
      "increment()": {
        "calls": 1,
        "min": 43482,
        "mean": 43482,
        "median": 43482,
        "max": 43482
      },
      "number()": {
        "calls": 1,
        "min": 424,
        "mean": 424,
        "median": 424,
        "max": 424
      },
      "setNumber(uint256)": {
        "calls": 1,
        "min": 23784,
        "mean": 23784,
        "median": 23784,
        "max": 23784
      }
    }
  },
  {
    "contract": "test/Counter.t.sol:CounterTest",
    "deployment": {
      "gas": 1545498,
      "size": 7578
    },
    "functions": {
      "setUp()": {
        "calls": 1,
        "min": 218902,
        "mean": 218902,
        "median": 218902,
        "max": 218902
      },
      "test_Increment()": {
        "calls": 1,
        "min": 52915,
        "mean": 52915,
        "median": 52915,
        "max": 52915
      }
    }
  }
]
"#]]
            .is_json(),
        );
});

forgetest_async!(gas_report_fuzz_invariant, |prj, cmd| {
    // speed up test by running with depth of 15
    prj.update_config(|config| config.invariant.depth = 15);

    prj.insert_ds_test();
    prj.add_source(
        "Contracts.sol",
        r#"
import "./test.sol";

contract Foo {
    function foo() public {}
}

contract Bar {
    function bar() public {}
}

contract FooBarTest is DSTest {
    Foo public targetContract;

    function setUp() public {
        targetContract = new Foo();
    }

    function invariant_dummy() public {
        assertTrue(true);
    }

    function testFuzz_bar(uint256 _val) public {
        (new Bar()).bar();
    }
}
    "#,
    )
    .unwrap();

    cmd.args(["test", "--gas-report"]).assert_success();
});

// <https://github.com/foundry-rs/foundry/issues/5847>
forgetest_init!(can_bind_enum_modules, |prj, cmd| {
    prj.clear();

    prj.add_source(
        "Enum.sol",
        r#"
    contract Enum {
        enum MyEnum { A, B, C }
    }
    "#,
    )
    .unwrap();

    prj.add_source(
        "UseEnum.sol",
        r#"
    import "./Enum.sol";
    contract UseEnum {
        Enum.MyEnum public myEnum;
    }"#,
    )
    .unwrap();

    cmd.args(["bind", "--select", "^Enum$"]).assert_success().stdout_eq(str![[
        r#"[COMPILING_FILES] with [SOLC_VERSION]
[SOLC_VERSION] [ELAPSED]
Compiler run successful!
Generating bindings for 1 contracts
Bindings have been generated to [..]"#
    ]]);
});
