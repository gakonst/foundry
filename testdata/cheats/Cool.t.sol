// SPDX-License-Identifier: Unlicense
pragma solidity 0.8.18;

import "../lib/ds-test/src/test.sol";
import "./Vm.sol";

contract CoolTest is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);
    uint256 public slot0;
    uint256 public slot1 = 1;
    MiniERC public erc;

    function setUp() public {
        erc = new MiniERC();
        erc.mint(address(1337), 1 ether);
    }

    function testCool_SLOAD_normal() public {
        uint256 startGas;
        uint256 endGas;
        uint256 beforeCoolGas;
        uint256 noCoolGas;

        startGas = gasleft();
        uint256 val = slot0;
        endGas = gasleft();
        beforeCoolGas = startGas - endGas;

        startGas = gasleft();
        uint256 val2 = slot0;
        endGas = gasleft();
        noCoolGas = startGas - endGas;

        assertEq(val, val2);
        assertGt(beforeCoolGas, noCoolGas);
    }

    function testCool_SLOAD() public {
        uint256 startGas;
        uint256 endGas;
        uint256 beforeCoolGas;
        uint256 afterCoolGas;
        uint256 warmGas;
        uint256 secondCoolGas;
        uint256 extraGas;

        startGas = gasleft();
        uint256 val = slot0;
        endGas = gasleft();
        beforeCoolGas = startGas - endGas;

        vm.cool(address(this));

        startGas = gasleft();
        uint256 val2 = slot0;
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        extraGas = afterCoolGas - 2100;

        assertEq(val, val2);
        assertEq(beforeCoolGas, afterCoolGas);
        assertEq(beforeCoolGas, 2100 + extraGas);

        startGas = gasleft();
        uint256 val3 = slot0;
        endGas = gasleft();
        warmGas = startGas - endGas;

        assertEq(val2, val3);
        assertGt(beforeCoolGas, warmGas);
        assertEq(warmGas, 100 + extraGas);

        // cool again to see if same resut
        vm.cool(address(this));

        startGas = gasleft();
        uint256 val4 = slot0;
        endGas = gasleft();
        secondCoolGas = startGas - endGas;

        assertEq(val, val4);
        assertEq(beforeCoolGas, secondCoolGas);
        assertEq(beforeCoolGas, 2100 + extraGas);
    }

    // check if slot value is preserved
    function testCool_SSTORE_check_slot_value() public {
        slot0 = 2;
        assertEq(slot0, 2);
        assertEq(slot1, 1);

        vm.cool(address(this));
        assertEq(slot0, 2);
        assertEq(slot1, 1);

        slot0 = 3;
        assertEq(slot0, 3);
        assertEq(slot1, 1);

        vm.cool(address(this));
        assertEq(slot0, 3);
        assertEq(slot1, 1);

        slot0 = 8;
        slot1 = 9;

        vm.cool(address(this));
        assertEq(slot0, 8);
        assertEq(slot1, 9);
    }

    function testCool_SSTORE_nonzero_to_nonzero() public {
        uint256 startGas;
        uint256 endGas;
        uint256 beforeCoolGas;
        uint256 afterCoolGas;
        uint256 warmGas;
        uint256 extraGas;

        // start as non-zero
        startGas = gasleft();
        slot1 = 2; // 5k gas
        endGas = gasleft();
        beforeCoolGas = startGas - endGas;
        extraGas = beforeCoolGas - 2900 - 2100;
        assertEq(slot1, 2);
        assertEq(beforeCoolGas, 2900 + 2100 + extraGas);

        // cool and set to same value
        vm.cool(address(this));

        startGas = gasleft();
        slot1 = 2; // 5k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot1, 2);
        assertEq(afterCoolGas, 100 + 2100 + extraGas);

        // cool and set from non-zero to another non-zero
        vm.cool(address(this));

        startGas = gasleft();
        slot1 = 3; // 5k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot1, 3);
        assertEq(afterCoolGas, 2900 + 2100 + extraGas);

        // don't cool and set non-zero to another non-zero
        startGas = gasleft();
        slot1 = 3; // 100 gas
        endGas = gasleft();
        warmGas = startGas - endGas;
        assertEq(slot1, 3);
        assertGt(afterCoolGas, warmGas);
        assertEq(warmGas, 100 + extraGas);

        // don't cool and set non-zero to another non-zero
        startGas = gasleft();
        slot1 = 4; // 100 gas
        endGas = gasleft();
        warmGas = startGas - endGas;
        assertEq(slot1, 4);
        assertGt(afterCoolGas, warmGas);
        assertEq(warmGas, 100 + extraGas);
    }

    function testCool_SSTORE_zero_to_nonzero() public {
        uint256 startGas;
        uint256 endGas;
        uint256 afterCoolGas;
        uint256 warmGas;
        uint256 extraGas;

        // start as zero
        // set from zero to non-zero
        startGas = gasleft();
        slot0 = 1; // 22.1k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        extraGas = afterCoolGas - 20000 - 2100;
        assertEq(slot0, 1);
        assertEq(afterCoolGas, 20000 + 2100 + extraGas);

        slot0 = 0;
        vm.cool(address(this));

        // set from zero to non-zero
        startGas = gasleft();
        slot0 = 1; // 22.1k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot0, 1);
        assertEq(afterCoolGas, 20000 + 2100 + extraGas);

        // don't cool and set non-zero to another non-zero
        startGas = gasleft();
        slot0 = 2; // 100
        endGas = gasleft();
        warmGas = startGas - endGas;
        assertEq(slot0, 2); // persisted state
        assertGt(afterCoolGas, warmGas);
        assertEq(warmGas, 100 + extraGas);

        // cool again
        // set from non-zero to non-zero
        vm.cool(address(this));
        startGas = gasleft();
        slot0 = 1; // 5k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot0, 1);
        assertEq(afterCoolGas, 2900 + 2100 + extraGas);

        // cool again, set to zero
        // set from zero to non-zero
        slot0 = 0;
        vm.cool(address(this));
        startGas = gasleft();
        slot0 = 1; // 22.1k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot0, 1);
        assertEq(afterCoolGas, 20000 + 2100 + extraGas);

        // cool again
        // set to same value
        vm.cool(address(this));
        startGas = gasleft();
        slot0 = 1; // 2.2k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot0, 1);
        assertEq(afterCoolGas, 100 + 2100 + extraGas);
    }

    function testCool_SSTORE_Multiple() public {
        uint256 startGas;
        uint256 endGas;
        uint256 afterCoolGas;
        uint256 extraGas;

        // start as zero
        assertEq(slot0, 0);

        vm.cool(address(this));
        vm.cool(address(this));

        // set from zero to non-zero
        startGas = gasleft();
        slot0 = 3; // 22.1k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        extraGas = afterCoolGas - 20000 - 2100;
        assertEq(slot0, 3);
        assertEq(afterCoolGas, 20000 + 2100 + extraGas);

        vm.cool(address(this));
        vm.cool(address(this));
        vm.cool(address(this));

        // set from non-zero to non-zero
        startGas = gasleft();
        slot0 = 2; // 5k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot0, 2);
        assertEq(afterCoolGas, 2900 + 2100 + extraGas);
    }

    function testCool_Once() public {
        uint256 startGas;
        uint256 endGas;
        uint256 afterCoolGas;
        uint256 extraGas;

        // start as zero
        assertEq(slot0, 0);
        vm.cool(address(this));

        // set from zero to non-zero
        startGas = gasleft();
        slot0 = 3; // 22.1k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot0, 3);
        extraGas = afterCoolGas - 20000 - 2100;

        // set from non-zero to non-zero
        startGas = gasleft();
        slot0 = 2; // 5k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot0, 2);
        assertEq(afterCoolGas, 100 + extraGas);

        // set to same
        startGas = gasleft();
        slot0 = 2; // 5k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot0, 2);
        assertEq(afterCoolGas, 100 + extraGas);

        // set from non-zero to non-zero
        startGas = gasleft();
        slot0 = 4; // 5k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot0, 4);
        assertEq(afterCoolGas, 100 + extraGas);

        // set from non-zero to zero
        startGas = gasleft();
        slot0 = 0; // 5k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot0, 0);
        assertEq(afterCoolGas, 100 + extraGas);

        // set from zero to non-zero
        startGas = gasleft();
        slot0 = 1; // 5k gas
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(slot0, 1);
        assertEq(afterCoolGas, 20000 + extraGas);
    }

    function testCool_call() public {
        uint256 startGas;
        uint256 endGas;
        uint256 afterCoolGas;

        TestContract test = new TestContract();

        // zero to 1 (20k) but slot is warm
        startGas = gasleft();
        test.setSlot0(1);
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(test.slot0(), 1);
        assertGt(afterCoolGas, 20000);

        test.setSlot0(0);
        vm.cool(address(test));

        // zero to 1 (20k) and slot is cold
        startGas = gasleft();
        test.setSlot0(2);
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(test.slot0(), 2);
        assertGt(afterCoolGas, 20000 + 2100);

        test.setSlot0(1);
        vm.cool(address(test));

        // 1 to 2 (2900) and slot is cold
        startGas = gasleft();
        test.setSlot0(2);
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(test.slot0(), 2);
        assertGt(afterCoolGas, 2900 + 2100);

        test.setSlot0(1);
        vm.cool(address(test));

        // 1 to 1 (100 gas) and slot is cold
        startGas = gasleft();
        test.setSlot0(1);
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(test.slot0(), 1);
        assertGt(afterCoolGas, 100 + 2100);

        test.setBoth(0);
        vm.cool(address(test));

        // both 0 to 1 (20k * 2)
        startGas = gasleft();
        test.setBoth(1);
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(test.slot0(), 1);
        assertEq(test.slot1(), 1);
        assertGt(afterCoolGas, 20000 * 2 + 2100 * 2);

        test.setSlot0(0);
        vm.cool(address(test));

        // slot0 from 0 to 2 (20k)
        // slot1 from 1 to 2 (2900)
        startGas = gasleft();
        test.setBoth(2);
        endGas = gasleft();
        afterCoolGas = startGas - endGas;
        assertEq(test.slot0(), 2);
        assertEq(test.slot1(), 2);
        assertGt(afterCoolGas, 20000 + 2900 + 2100 * 2);
    }

    function testCool_Mint() public {
        uint256 startGas;
        uint256 endGas;
        uint256 beforeGas;

        startGas = gasleft();
        erc.mint(address(1337), 0.01 ether); // 15462
        endGas = gasleft();
        beforeGas = startGas - endGas;

        vm.cool(address(erc));
        vm.cool(address(this));

        startGas = gasleft();
        erc.mint(address(1337), 0.01 ether); // 15474
        endGas = gasleft();
        assertEq(beforeGas, startGas - endGas + 12); // ?
        beforeGas = startGas - endGas;

        vm.cool(address(erc));
        vm.cool(address(this));

        startGas = gasleft();
        erc.mint(address(1337), 0.01 ether); // 15474
        endGas = gasleft();
        assertEq(beforeGas, startGas - endGas);

        startGas = gasleft();
        erc.mint(address(1337), 0.01 ether); // 1362
        endGas = gasleft();
        assertLt(startGas - endGas, beforeGas);
    }
}

contract TestContract {
    uint256 public slot0 = 0;
    uint256 public slot1 = 1;

    function setSlot0(uint256 num) public {
        slot0 = num;
    }

    function setSlot1(uint256 num) public {
        slot1 = num;
    }

    function setBoth(uint256 num) public {
        slot0 = num;
        slot1 = num;
    }
}

contract MiniERC {
    mapping(address => uint256) private _balances;
    uint256 private _totalSupply;

    function mint(address to, uint256 amount) external {
        _totalSupply += amount;
        unchecked {
            _balances[to] += amount;
        }
    }
}
