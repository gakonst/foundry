// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.13;

import "ds-test/test.sol";

contract Pay {
    uint256 private counter;
    bool public found; // CBA with 2,msg.value>0.1234,0

    function A(uint8 x) external {
        if (counter == 2 && x == 0) found = true;
        else counter = 0;
    }
    function B() external payable {
        if (counter == 1 && msg.value > 0.1234 ether) counter++;
        else counter = 0;
    }
    function C(uint8 x) external {
        if (counter == 0 && x == 2) counter++;
    }
}

contract InvariantMsgValue is DSTest {
    Pay target;

    function setUp() public {
        target = new Pay();
    }

    /// forge-config: default.invariant.runs = 2000
    function invariant_msg_value_not_found() public view {
        require(!target.found(), "CBA with 2,msg.value>0.1234,0 found");
    }
}

