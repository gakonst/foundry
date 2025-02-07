// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract VariableMixedCaseTest {
    // Passes
    uint256 variableMixedCase;
    uint256 _variableMixedCase;

    // Fails
    uint256 Variablemixedcase;
    uint256 VARIABLE_MIXED_CASE;
    uint256 variablemixedcase;
    uint256 VariableMixedCase;

    function foo() public {
        // Passes
        uint256 testVal;
        uint256 testVAL;
        uint256 testVal123;

        // Fails
        uint256 TestVal;
        uint256 TESTVAL;
    }
}
