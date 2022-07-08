//! Contains various tests for checking forge's commands
use ethers::solc::{
    artifacts::{BytecodeHash, Metadata},
    ConfigurableContractArtifact,
};
use foundry_cli_test_utils::{
    ethers_solc::PathStyle,
    forgetest, forgetest_init,
    util::{pretty_err, read_string, remapping_str, OutputExt, TestCommand, TestProject},
};
use foundry_config::{parse_with_profile, BasicConfig, Chain, Config, SolidityErrorCode};
use std::{env, fs, path::PathBuf};

// tests `--help` is printed to std out
forgetest!(print_help, |_: TestProject, mut cmd: TestCommand| {
    cmd.arg("--help");
    cmd.assert_non_empty_stdout();
});

// checks that `clean` can be invoked even if out and cache don't exist
forgetest!(can_clean_non_existing, |prj: TestProject, mut cmd: TestCommand| {
    cmd.arg("clean");
    cmd.assert_empty_stdout();
    prj.assert_cleaned();
});

// checks that `cache ls` can be invoked and displays the foundry cache
forgetest!(
    #[ignore]
    can_cache_ls,
    |_: TestProject, mut cmd: TestCommand| {
        let chain = Chain::Named(ethers::prelude::Chain::Mainnet);
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

        cmd.args(["cache", "ls"]);
        let output_string = String::from_utf8_lossy(&cmd.output().stdout).to_string();
        let output_lines = output_string.split('\n').collect::<Vec<_>>();
        println!("{output_string}");

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
    |_: TestProject, mut cmd: TestCommand| {
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
    |_: TestProject, mut cmd: TestCommand| {
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
    |_: TestProject, mut cmd: TestCommand| {
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
    |_: TestProject, mut cmd: TestCommand| {
        let chain = Chain::Named(ethers::prelude::Chain::Mainnet);
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
    |_: TestProject, mut cmd: TestCommand| {
        let chain = Chain::Named(ethers::prelude::Chain::Mainnet);
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
    |_: TestProject, mut cmd: TestCommand| {
        let cache_dir =
            Config::foundry_chain_cache_dir(Chain::Named(ethers::prelude::Chain::Mainnet)).unwrap();
        let etherscan_cache_dir = Config::foundry_etherscan_chain_cache_dir(Chain::Named(
            ethers::prelude::Chain::Mainnet,
        ))
        .unwrap();
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
forgetest!(can_init_repo_with_config, |prj: TestProject, mut cmd: TestCommand| {
    let foundry_toml = prj.root().join(Config::FILE_NAME);
    assert!(!foundry_toml.exists());

    cmd.args(["init", "--force"]).arg(prj.root());
    cmd.assert_non_empty_stdout();

    let s = read_string(&foundry_toml);
    let _config: BasicConfig = parse_with_profile(&s).unwrap().unwrap().1;
});

// checks that init works repeatedly
forgetest!(can_init_repo_repeatedly_with_force, |prj: TestProject, mut cmd: TestCommand| {
    let foundry_toml = prj.root().join(Config::FILE_NAME);
    assert!(!foundry_toml.exists());

    prj.wipe();

    cmd.arg("init").arg(prj.root());
    cmd.assert_non_empty_stdout();

    cmd.arg("--force");

    for _ in 0..2 {
        assert!(foundry_toml.exists());
        pretty_err(&foundry_toml, fs::remove_file(&foundry_toml));
        cmd.assert_non_empty_stdout();
    }
});

// Checks that a forge project can be initialized without creating a git repository
forgetest!(can_init_no_git, |prj: TestProject, mut cmd: TestCommand| {
    prj.wipe();

    cmd.arg("init").arg(prj.root()).arg("--no-git");
    cmd.assert_non_empty_stdout();
    prj.assert_config_exists();

    assert!(!prj.root().join(".git").exists());
    assert!(prj.root().join("lib/forge-std").exists());
    assert!(!prj.root().join("lib/forge-std/.git").exists());
});

// Checks that quiet mode does not print anything
forgetest!(can_init_quiet, |prj: TestProject, mut cmd: TestCommand| {
    prj.wipe();

    cmd.arg("init").arg(prj.root()).arg("-q");
    let _ = cmd.output();
});

// `forge init` does only work on non-empty dirs
forgetest!(can_init_non_empty, |prj: TestProject, mut cmd: TestCommand| {
    prj.create_file("README.md", "non-empty dir");
    cmd.arg("init").arg(prj.root());
    cmd.assert_err();

    cmd.arg("--force");
    cmd.assert_non_empty_stdout();
    assert!(prj.root().join(".git").exists());
    assert!(prj.root().join("lib/forge-std").exists());
});

// Checks that remappings.txt and .vscode/settings.json is generated
forgetest!(can_init_vscode, |prj: TestProject, mut cmd: TestCommand| {
    prj.wipe();

    cmd.arg("init").arg(prj.root()).arg("--vscode");
    cmd.assert_non_empty_stdout();

    let settings = prj.root().join(".vscode/settings.json");
    assert!(settings.is_file());
    let settings: serde_json::Value = ethers::solc::utils::read_json_file(&settings).unwrap();
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
    assert_eq!(
        content,
        format!(
            "{}\n{}",
            remapping_str("ds-test/", "lib/forge-std/lib/ds-test/src"),
            remapping_str("forge-std/", "lib/forge-std/src")
        )
    );
});

// checks that forge can init with template
forgetest!(can_init_template, |prj: TestProject, mut cmd: TestCommand| {
    prj.wipe();
    cmd.args(["init", "--template", "foundry-rs/forge-template"]).arg(prj.root());
    cmd.assert_non_empty_stdout();
    assert!(prj.root().join(".git").exists());
    assert!(prj.root().join("foundry.toml").exists());
    assert!(prj.root().join("lib/forge-std").exists());
    assert!(prj.root().join("src").exists());
    assert!(prj.root().join("test").exists());
});

// checks that init fails when the provided template doesn't exist
forgetest!(fail_init_nonexistent_template, |prj: TestProject, mut cmd: TestCommand| {
    prj.wipe();
    cmd.args(["init", "--template", "a"]).arg(prj.root());
    cmd.assert_non_empty_stderr();
});

// checks that `clean` removes dapptools style paths
forgetest!(can_clean, |prj: TestProject, mut cmd: TestCommand| {
    prj.assert_create_dirs_exists();
    prj.assert_style_paths_exist(PathStyle::Dapptools);
    cmd.arg("clean");
    cmd.assert_empty_stdout();
    prj.assert_cleaned();
});

// checks that `clean` removes hardhat style paths
forgetest!(can_clean_hardhat, PathStyle::HardHat, |prj: TestProject, mut cmd: TestCommand| {
    prj.assert_create_dirs_exists();
    prj.assert_style_paths_exist(PathStyle::HardHat);
    cmd.arg("clean");
    cmd.assert_empty_stdout();
    prj.assert_cleaned();
});

// checks that `clean` also works with the "out" value set in Config
forgetest_init!(can_clean_config, |prj: TestProject, mut cmd: TestCommand| {
    let config = Config { out: "custom-out".into(), ..Default::default() };
    prj.write_config(config);
    cmd.arg("build");
    cmd.assert_non_empty_stdout();

    // default test contract is written in custom out directory
    let artifact = prj.root().join("custom-out/Contract.t.sol/ContractTest.json");
    assert!(artifact.exists());

    cmd.forge_fuse().arg("clean");
    cmd.output();
    assert!(!artifact.exists());
});

// checks that extra output works
forgetest_init!(can_emit_extra_output, |prj: TestProject, mut cmd: TestCommand| {
    cmd.args(["build", "--extra-output", "metadata"]);
    cmd.assert_non_empty_stdout();

    let artifact_path = prj.paths().artifacts.join("Contract.sol/Contract.json");
    let artifact: ConfigurableContractArtifact =
        ethers::solc::utils::read_json_file(artifact_path).unwrap();
    assert!(artifact.metadata.is_some());

    cmd.forge_fuse().args(["build", "--extra-output-files", "metadata", "--force"]).root_arg();
    cmd.assert_non_empty_stdout();

    let metadata_path = prj.paths().artifacts.join("Contract.sol/Contract.metadata.json");
    let _artifact: Metadata = ethers::solc::utils::read_json_file(metadata_path).unwrap();
});

// checks that extra output works
forgetest_init!(can_emit_multiple_extra_output, |prj: TestProject, mut cmd: TestCommand| {
    cmd.args(["build", "--extra-output", "metadata", "ir-optimized", "--extra-output", "ir"]);
    cmd.assert_non_empty_stdout();

    let artifact_path = prj.paths().artifacts.join("Contract.sol/Contract.json");
    let artifact: ConfigurableContractArtifact =
        ethers::solc::utils::read_json_file(artifact_path).unwrap();
    assert!(artifact.metadata.is_some());
    assert!(artifact.ir.is_some());
    assert!(artifact.ir_optimized.is_some());

    cmd.forge_fuse()
        .args([
            "build",
            "--extra-output-files",
            "metadata",
            "ir-optimized",
            "evm.bytecode.sourceMap",
            "--force",
        ])
        .root_arg();
    cmd.assert_non_empty_stdout();

    let metadata_path = prj.paths().artifacts.join("Contract.sol/Contract.metadata.json");
    let _artifact: Metadata = ethers::solc::utils::read_json_file(metadata_path).unwrap();

    let iropt = prj.paths().artifacts.join("Contract.sol/Contract.iropt");
    std::fs::read_to_string(iropt).unwrap();

    let sourcemap = prj.paths().artifacts.join("Contract.sol/Contract.sourcemap");
    std::fs::read_to_string(sourcemap).unwrap();
});

forgetest!(can_print_warnings, |prj: TestProject, mut cmd: TestCommand| {
    prj.inner()
        .add_source(
            "Foo",
            r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity >0.8.9;
contract Greeter {
    function foo(uint256 a) public {
        uint256 x = 1;
    }
}
   "#,
        )
        .unwrap();

    // explicitly set to run with 0.8.10
    let config = Config { solc: Some("0.8.10".into()), ..Default::default() };
    prj.write_config(config);

    cmd.arg("build");

    let output = cmd.stdout_lossy();
    assert!(output.contains(
        "
Compiler run successful (with warnings)
warning[5667]: Warning: Unused function parameter. Remove or comment out the variable name to silence this warning.
",
    ));
});

// Tests that direct import paths are handled correctly
//
// NOTE(onbjerg): Disabled for Windows -- for some reason solc fails with a bogus error message
// here: error[9553]: TypeError: Invalid type for argument in function call. Invalid implicit
// conversion from struct Bar memory to struct Bar memory requested.   --> src\Foo.sol:12:22:
//    |
// 12 |         FooLib.check(b);
//    |                      ^
//
//
//
// error[9553]: TypeError: Invalid type for argument in function call. Invalid implicit conversion
// from contract Foo to contract Foo requested.   --> src\Foo.sol:15:23:
//    |
// 15 |         FooLib.check2(this);
//    |                       ^^^^
#[cfg(not(target_os = "windows"))]
forgetest!(can_handle_direct_imports_into_src, |prj: TestProject, mut cmd: TestCommand| {
    prj.inner()
        .add_source(
            "Foo",
            r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.10;
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

    prj.inner()
        .add_source(
            "FooLib",
            r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.10;
import {Foo, Bar} from "src/Foo.sol";
library FooLib {
    function check(Bar memory b) public {}
    function check2(Foo f) public {}
}
   "#,
        )
        .unwrap();

    cmd.arg("build");

    assert!(cmd.stdout_lossy().ends_with(
        "
Compiler run successful
"
    ));
});

// tests that the `inspect` command works correctly
forgetest!(can_execute_inspect_command, |prj: TestProject, mut cmd: TestCommand| {
    // explicitly set to include the ipfs bytecode hash
    let config = Config { bytecode_hash: BytecodeHash::Ipfs, ..Default::default() };
    prj.write_config(config);
    let contract_name = "Foo";
    let _ = prj
        .inner()
        .add_source(
            contract_name,
            r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.10;
contract Foo {
    event log_string(string);
    function run() external {
        emit log_string("script ran");
    }
}
    "#,
        )
        .unwrap();

    // Remove the ipfs hash from the metadata
    let mut dynamic_bytecode = "0x608060405234801561001057600080fd5b5060c08061001f6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063c040622614602d575b600080fd5b60336035565b005b7f0b2e13ff20ac7b474198655583edf70dedd2c1dc980e329c4fbb2fc0748b796b6040516080906020808252600a908201526939b1b934b83a103930b760b11b604082015260600190565b60405180910390a156fea264697066735822122065c066d19101ad1707272b9a884891af8ab0cf5a0e0bba70c4650594492c14be64736f6c634300080a0033\n".to_string();
    let ipfs_start = dynamic_bytecode.len() - (24 + 64);
    let ipfs_end = ipfs_start + 65;
    dynamic_bytecode.replace_range(ipfs_start..ipfs_end, "");
    cmd.arg("inspect").arg(contract_name).arg("bytecode");
    let mut output = cmd.stdout_lossy();
    output.replace_range(ipfs_start..ipfs_end, "");

    // Compare the static bytecode
    assert_eq!(dynamic_bytecode, output);
});

// test that `forge snapshot` commands work
forgetest!(
    #[serial_test::serial]
    can_check_snapshot,
    |prj: TestProject, mut cmd: TestCommand| {
        prj.insert_ds_test();

        prj.inner()
            .add_source(
                "ATest.t.sol",
                r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.10;
import "./test.sol";
contract ATest is DSTest {
    function testExample() public {
        assertTrue(true);
    }
}
   "#,
            )
            .unwrap();

        cmd.arg("snapshot");

        cmd.unchecked_output().stdout_matches_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/can_check_snapshot.stdout"),
        );

        cmd.arg("--check");
        let _ = cmd.output();
    }
);

// test that `forge build` does not print `(with warnings)` if there arent any
forgetest!(can_compile_without_warnings, |prj: TestProject, mut cmd: TestCommand| {
    let config = Config {
        ignored_error_codes: vec![SolidityErrorCode::SpdxLicenseNotProvided],
        ..Default::default()
    };
    prj.write_config(config);
    prj.inner()
        .add_source(
            "A",
            r#"
pragma solidity 0.8.10;
contract A {
    function testExample() public {}
}
   "#,
        )
        .unwrap();

    cmd.args(["build", "--force"]);
    let out = cmd.stdout();
    // no warnings
    assert!(out.trim().contains("Compiler run successful"));
    assert!(!out.trim().contains("Compiler run successful (with warnings)"));

    // don't ignore errors
    let config = Config { ignored_error_codes: vec![], ..Default::default() };
    prj.write_config(config);
    let out = cmd.stdout();

    assert!(out.trim().contains("Compiler run successful (with warnings)"));
    assert!(
      out.contains(
                    r#"Warning: SPDX license identifier not provided in source file. Before publishing, consider adding a comment containing "SPDX-License-Identifier: <SPDX-License>" to each source file. Use "SPDX-License-Identifier: UNLICENSED" for non-open-source code. Please see https://spdx.org for more information."#
        )
    );
});

// test against a local checkout, useful to debug with local ethers-rs patch
forgetest!(
    #[ignore]
    can_compile_local_spells,
    |_: TestProject, mut cmd: TestCommand| {
        let current_dir = std::env::current_dir().unwrap();
        let root = current_dir
            .join("../../foundry-integration-tests/testdata/spells-mainnet")
            .to_string_lossy()
            .to_string();
        println!("project root: \"{root}\"");

        let eth_rpc_url = foundry_utils::rpc::next_http_archive_rpc_endpoint();
        let dss_exec_lib = "src/DssSpell.sol:DssExecLib:0xfD88CeE74f7D78697775aBDAE53f9Da1559728E4";

        cmd.args([
            "test",
            "--root",
            root.as_str(),
            "--fork-url",
            eth_rpc_url.as_str(),
            "--fork-block-number",
            "14435000",
            "--libraries",
            dss_exec_lib,
            "-vvv",
        ]);
        cmd.print_output();
    }
);

// test that a failing `forge build` does not impact followup builds
forgetest!(can_build_after_failure, |prj: TestProject, mut cmd: TestCommand| {
    prj.insert_ds_test();

    prj.inner()
        .add_source(
            "ATest.t.sol",
            r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.10;
import "./test.sol";
contract ATest is DSTest {
    function testExample() public {
        assertTrue(true);
    }
}
   "#,
        )
        .unwrap();
    prj.inner()
        .add_source(
            "BTest.t.sol",
            r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.10;
import "./test.sol";
contract BTest is DSTest {
    function testExample() public {
        assertTrue(true);
    }
}
   "#,
        )
        .unwrap();

    cmd.arg("build");
    cmd.assert_non_empty_stdout();
    prj.assert_cache_exists();
    prj.assert_artifacts_dir_exists();

    let syntax_err = r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.10;
import "./test.sol";
contract CTest is DSTest {
    function testExample() public {
        THIS WILL CAUSE AN ERROR
    }
}
   "#;

    // introduce contract with syntax error
    prj.inner().add_source("CTest.t.sol", syntax_err).unwrap();

    // `forge build --force` which should fail
    cmd.arg("--force");
    cmd.assert_err();

    // but ensure this cleaned cache and artifacts
    assert!(!prj.paths().artifacts.exists());
    assert!(!prj.cache_path().exists());

    // still errors
    cmd.forge_fuse().arg("build");
    cmd.assert_err();

    // resolve the error by replacing the file
    prj.inner()
        .add_source(
            "CTest.t.sol",
            r#"
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.10;
import "./test.sol";
contract CTest is DSTest {
    function testExample() public {
         assertTrue(true);
    }
}
   "#,
        )
        .unwrap();

    cmd.assert_non_empty_stdout();
    prj.assert_cache_exists();
    prj.assert_artifacts_dir_exists();

    // ensure cache is unchanged after error
    let cache = fs::read_to_string(prj.cache_path()).unwrap();

    // introduce the error again but building without force
    prj.inner().add_source("CTest.t.sol", syntax_err).unwrap();
    cmd.assert_err();

    // ensure unchanged cache file
    let cache_after = fs::read_to_string(prj.cache_path()).unwrap();
    assert_eq!(cache, cache_after);
});

// test to check that install/remove works properly
forgetest!(can_install_and_remove, |prj: TestProject, mut cmd: TestCommand| {
    cmd.git_init();

    let libs = prj.root().join("lib");
    let git_mod = prj.root().join(".git/modules/lib");
    let git_mod_file = prj.root().join(".gitmodules");

    let forge_std = libs.join("forge-std");
    let forge_std_mod = git_mod.join("forge-std");

    let install = |cmd: &mut TestCommand| {
        cmd.forge_fuse().args(["install", "foundry-rs/forge-std", "--no-commit"]);
        cmd.assert_non_empty_stdout();
        assert!(forge_std.exists());
        assert!(forge_std_mod.exists());

        let submods = read_string(&git_mod_file);
        assert!(submods.contains("https://github.com/foundry-rs/forge-std"));
    };

    let remove = |cmd: &mut TestCommand, target: &str| {
        cmd.forge_fuse().args(["remove", target]);
        cmd.assert_non_empty_stdout();
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

// test to check that package can be reinstalled after manually removing the directory
forgetest!(can_reinstall_after_manual_remove, |prj: TestProject, mut cmd: TestCommand| {
    cmd.git_init();

    let libs = prj.root().join("lib");
    let git_mod = prj.root().join(".git/modules/lib");
    let git_mod_file = prj.root().join(".gitmodules");

    let forge_std = libs.join("forge-std");
    let forge_std_mod = git_mod.join("forge-std");

    let install = |cmd: &mut TestCommand| {
        cmd.forge_fuse().args(["install", "foundry-rs/forge-std", "--no-commit"]);
        cmd.assert_non_empty_stdout();
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
