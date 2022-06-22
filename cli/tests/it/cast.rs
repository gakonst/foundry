//! Contains various tests for checking cast commands

use foundry_cli_test_utils::{
    casttest,
    util::{TestCommand, TestProject},
};
use foundry_utils::rpc::next_http_rpc_endpoint;
use std::{io::Write, path::PathBuf};

// tests that the `cast find-block` command works correctly
casttest!(finds_block, |_: TestProject, mut cmd: TestCommand| {
    // Construct args
    let timestamp = "1647843609".to_string();
    let eth_rpc_url = next_http_rpc_endpoint();

    // Call `cast find-block`
    cmd.args(["find-block", "--rpc-url", eth_rpc_url.as_str(), &timestamp]);
    let output = cmd.stdout_lossy();
    println!("{output}");

    // Expect successful block query
    // Query: 1647843609, Mar 21 2022 06:20:09 UTC
    // Output block: https://etherscan.io/block/14428082
    // Output block time: Mar 21 2022 06:20:09 UTC
    assert!(output.contains("14428082"), "{}", output);
});

// tests that we can create a new wallet with keystore
casttest!(new_wallet_keystore_with_password, |_: TestProject, mut cmd: TestCommand| {
    cmd.args(["wallet", "new", ".", "--unsafe-password", "test"]);
    let out = cmd.stdout_lossy();
    assert!(out.contains("Created new encrypted keystore file"));
    assert!(out.contains("Public Address of the key"));
});

// tests that the `cast upload-signatures` command works correctly
casttest!(upload_signatures, |_: TestProject, mut cmd: TestCommand| {
    // test no prefix is accepted as function
    cmd.args(["upload-signature", "transfer(address,uint256)"]);
    let output = cmd.stdout_lossy();

    assert!(output.contains("Function transfer(address,uint256): 0xa9059cbb"), "{}", output);

    // test event prefix
    cmd.args(["upload-signature", "event Transfer(address,uint256)"]);
    let output = cmd.stdout_lossy();

    assert!(output.contains("Event Transfer(address,uint256): 0x69ca02dd4edd7bf0a4abb9ed3b7af3f14778db5d61921c7dc7cd545266326de2"), "{}", output);

    // test multiple sigs
    cmd.args([
        "upload-signature",
        "event Transfer(address,uint256)",
        "transfer(address,uint256)",
        "approve(address,uint256)",
    ]);
    let output = cmd.stdout_lossy();

    assert!(output.contains("Event Transfer(address,uint256): 0x69ca02dd4edd7bf0a4abb9ed3b7af3f14778db5d61921c7dc7cd545266326de2"), "{}", output);
    assert!(output.contains("Function transfer(address,uint256): 0xa9059cbb"), "{}", output);
    assert!(output.contains("Function approve(address,uint256): 0x095ea7b3"), "{}", output);

    // test abi
    cmd.args([
        "upload-signature",
        "event Transfer(address,uint256)",
        "transfer(address,uint256)",
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/ERC20Artifact.json")
            .into_os_string()
            .into_string()
            .unwrap()
            .as_str(),
    ]);
    let output = cmd.stdout_lossy();

    assert!(output.contains("Event Transfer(address,uint256): 0x69ca02dd4edd7bf0a4abb9ed3b7af3f14778db5d61921c7dc7cd545266326de2"), "{}", output);
    assert!(output.contains("Function transfer(address,uint256): 0xa9059cbb"), "{}", output);
    assert!(output.contains("Function approve(address,uint256): 0x095ea7b3"), "{}", output);
    assert!(output.contains("Function decimals(): 0x313ce567"), "{}", output);
    assert!(output.contains("Function allowance(address,address): 0xdd62ed3e"), "{}", output);
});

// tests that the `cast to-rlp` and `cast from-rlp` commands work correctly
casttest!(cast_rlp, |_: TestProject, mut cmd: TestCommand| {
    cmd.args(["--to-rlp", "[\"0xaa\", [[\"bb\"]], \"0xcc\"]"]);
    let out = cmd.stdout_lossy();
    assert!(out.contains("0xc881aac3c281bb81cc"), "{}", out);

    cmd.cast_fuse();
    cmd.args(["--from-rlp", "0xcbc58455556666c0c0c2c1c0"]);
    let out = cmd.stdout_lossy();
    assert!(out.contains("[[\"0x55556666\"],[],[],[[[]]]]"), "{}", out);
});

// test for cast_rpc without arguments
casttest!(cast_rpc_no_args, |_: TestProject, mut cmd: TestCommand| {
    let eth_rpc_url = next_http_rpc_endpoint();

    // Call `cast rpc eth_chainId`
    cmd.args(["rpc", "--rpc-url", eth_rpc_url.as_str(), "eth_chainId"]);
    let output = cmd.stdout_lossy();
    assert_eq!(output.trim_end(), r#""0x1""#);
});

// test for cast_rpc with arguments
casttest!(cast_rpc_with_args, |_: TestProject, mut cmd: TestCommand| {
    let eth_rpc_url = next_http_rpc_endpoint();

    // Call `cast rpc eth_getBlockByNumber 0x123 false`
    cmd.args(["rpc", "--rpc-url", eth_rpc_url.as_str(), "eth_getBlockByNumber", "0x123", "false"]);
    let output = cmd.stdout_lossy();
    assert!(output.contains(r#""number":"0x123""#), "{}", output);
});

// test for cast_rpc with raw params
casttest!(cast_rpc_raw_params, |_: TestProject, mut cmd: TestCommand| {
    let eth_rpc_url = next_http_rpc_endpoint();

    // Call `cast rpc eth_getBlockByNumber --raw '["0x123", false]'`
    cmd.args([
        "rpc",
        "--rpc-url",
        eth_rpc_url.as_str(),
        "eth_getBlockByNumber",
        "--raw",
        r#"["0x123", false]"#,
    ]);
    let output = cmd.stdout_lossy();
    assert!(output.contains(r#""number":"0x123""#), "{}", output);
});

// test for cast_rpc with direct params
casttest!(cast_rpc_raw_params_stdin, |_: TestProject, mut cmd: TestCommand| {
    let eth_rpc_url = next_http_rpc_endpoint();

    // Call `echo "\n[\n\"0x123\",\nfalse\n]\n" | cast rpc  eth_getBlockByNumber --raw
    cmd.args(["rpc", "--rpc-url", eth_rpc_url.as_str(), "eth_getBlockByNumber", "--raw"]).stdin(
        |mut stdin| {
            stdin.write_all(b"\n[\n\"0x123\",\nfalse\n]\n").unwrap();
        },
    );
    let output = cmd.stdout_lossy();
    assert!(output.contains(r#""number":"0x123""#), "{}", output);
});
