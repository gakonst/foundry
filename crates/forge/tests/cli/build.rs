use foundry_config::Config;
use foundry_test_utils::forgetest;
use globset::Glob;
use snapbox::file;

// tests that json is printed when --json is passed
forgetest!(compile_json, |prj, cmd| {
    prj.add_source(
        "jsonError",
        r"
contract Dummy {
    uint256 public number;
    function something(uint256 newNumber) public {
        number = newnumber; // error here
    }
}
",
    )
    .unwrap();

    // set up command
    cmd.args(["compile", "--format-json"])
        .assert()
        .stdout_eq(file!["../fixtures/compile_json.stdout": Json]);
});

// tests build output is as expected
forgetest_init!(exact_build_output, |prj, cmd| {
    cmd.args(["build", "--force"]).assert_success().stdout_eq("Compiling[..]\n...");
});

// tests build output is as expected
forgetest_init!(build_sizes_no_forge_std, |prj, cmd| {
    cmd.args(["build", "--sizes"]);
    let stdout = cmd.stdout_lossy();
    assert!(!stdout.contains("console"), "\n{stdout}");
    assert!(!stdout.contains("std"), "\n{stdout}");
    assert!(stdout.contains("Counter"), "\n{stdout}");
});

// tests that skip key in config can be used to skip non-compilable contract
forgetest_init!(test_can_skip_contract, |prj, cmd| {
    prj.add_source(
        "InvalidContract",
        r"
contract InvalidContract {
    some_invalid_syntax
}
",
    )
    .unwrap();

    prj.add_source(
        "ValidContract",
        r"
contract ValidContract {}
",
    )
    .unwrap();

    let config =
        Config { skip: vec![Glob::new("src/InvalidContract.sol").unwrap()], ..Default::default() };
    prj.write_config(config);

    cmd.args(["build"]).assert_success();
});
