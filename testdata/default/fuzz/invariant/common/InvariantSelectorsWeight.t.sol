// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "ds-test/test.sol";

contract HandlerOne {
    uint256 public hit1;

    function selector1() external {
        hit1 += 1;
    }
}

contract HandlerTwo {
    uint256 public hit2;
    uint256 public hit3;
    uint256 public hit4;
    uint256 public hit5;

    function selector2() external {
        hit2 += 1;
    }

    function selector3() external {
        hit3 += 1;
    }

    function selector4() external {
        hit4 += 1;
    }

    function selector5() external {
        hit5 += 1;
    }
}

contract InvariantSelectorsWeightTest is DSTest {
    HandlerOne handlerOne;
    HandlerTwo handlerTwo;

    function setUp() public {
        handlerOne = new HandlerOne();
        handlerTwo = new HandlerTwo();
    }

    function afterInvariant() public {
        // selector hits uniformly distributed, see https://github.com/foundry-rs/foundry/issues/2986
        uint256[5] memory hits =
            [handlerOne.hit1(), handlerTwo.hit2(), handlerTwo.hit3(), handlerTwo.hit4(), handlerTwo.hit5()];

        uint256 hits_sum;
        for (uint256 i = 0; i < hits.length; i++) {
            hits_sum += hits[i];
        }
        uint256 average = (hits_sum) / hits.length;
        for (uint256 i = 0; i < hits.length; i++) {
            uint256 delta = average > hits[i] ? average - hits[i] : hits[i] - average;
            uint256 delta_scaled = delta * 100 / average;
            require(delta_scaled <= 10, "Selectors Delta > 10%");
        }
    }

    function invariant_selectors_weight() public view {}
}
