// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.18;

import "ds-test/test.sol";
import "cheats/Vm.sol";

contract GasSnapshotIsolatedTest is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    uint256 public slot0;
    Flare public flare;

    function setUp() public {
        flare = new Flare();
    }

    function testSnapshotGasSectionExternal() public {
        vm.startSnapshotGas("testAssertGasExternal");
        flare.run(1);
        uint256 gasUsed = vm.stopSnapshotGas();

        assertGt(gasUsed, 0);
    }

    function testSnapshotGasSectionInternal() public {
        vm.startSnapshotGas("testAssertGasInternalA");
        slot0 = 1;
        vm.stopSnapshotGas();

        vm.startSnapshotGas("testAssertGasInternalB");
        slot0 = 2;
        vm.stopSnapshotGas();

        vm.startSnapshotGas("testAssertGasInternalC");
        slot0 = 0;
        vm.stopSnapshotGas();

        vm.startSnapshotGas("testAssertGasInternalD");
        slot0 = 1;
        vm.stopSnapshotGas();

        vm.startSnapshotGas("testAssertGasInternalE");
        slot0 = 2;
        vm.stopSnapshotGas();
    }

    // Writes to `GasSnapshotTest` group with custom names.
    function testSnapshotValueDefaultGroupA() public {
        uint256 a = 123;
        uint256 b = 456;
        uint256 c = 789;

        vm.snapshotValue("a", a);
        vm.snapshotValue("b", b);
        vm.snapshotValue("c", c);
    }

    // Writes to same `GasSnapshotTest` group with custom names.
    function testSnapshotValueDefaultGroupB() public {
        uint256 d = 123;
        uint256 e = 456;
        uint256 f = 789;

        vm.snapshotValue("d", d);
        vm.snapshotValue("e", e);
        vm.snapshotValue("f", f);
    }

    // Writes to `CustomGroup` group with custom names.
    // Asserts that the order of the values is alphabetical.
    function testSnapshotValueCustomGroupA() public {
        uint256 o = 123;
        uint256 i = 456;
        uint256 q = 789;

        vm.snapshotValue("CustomGroup", "q", q);
        vm.snapshotValue("CustomGroup", "i", i);
        vm.snapshotValue("CustomGroup", "o", o);
    }

    // Writes to `CustomGroup` group with custom names.
    // Asserts that the order of the values is alphabetical.
    function testSnapshotValueCustomGroupB() public {
        uint256 x = 123;
        uint256 e = 456;
        uint256 z = 789;

        vm.snapshotValue("CustomGroup", "z", z);
        vm.snapshotValue("CustomGroup", "x", x);
        vm.snapshotValue("CustomGroup", "e", e);
    }

    // Writes to `GasSnapshotTest` group with `testSnapshotGasDefault` name.
    function testSnapshotGasSectionDefaultGroupStop() public {
        vm.startSnapshotGas("testSnapshotGasSection");

        flare.run(256);

        // vm.stopSnapshotGas() will use the last snapshot name.
        uint256 gasUsed = vm.stopSnapshotGas();
        assertGt(gasUsed, 0);
    }

    // Writes to `GasSnapshotTest` group with `testSnapshotGasCustom` name.
    function testSnapshotGasSectionCustomGroupStop() public {
        vm.startSnapshotGas("CustomGroup", "testSnapshotGasSection");

        flare.run(256);

        // vm.stopSnapshotGas() will use the last snapshot name, even with custom group.
        uint256 gasUsed = vm.stopSnapshotGas();
        assertGt(gasUsed, 0);
    }

    // Writes to `GasSnapshotTest` group with `testSnapshotGasSection` name.
    function testSnapshotGasSectionName() public {
        vm.startSnapshotGas("testSnapshotGasSectionName");

        flare.run(256);

        uint256 gasUsed = vm.stopSnapshotGas("testSnapshotGasSectionName");
        assertGt(gasUsed, 0);
    }

    // Writes to `CustomGroup` group with `testSnapshotGasSection` name.
    function testSnapshotGasSectionGroupName() public {
        vm.startSnapshotGas("CustomGroup", "testSnapshotGasSectionGroupName");

        flare.run(256);

        uint256 gasUsed = vm.stopSnapshotGas("CustomGroup", "testSnapshotGasSectionGroupName");
        assertGt(gasUsed, 0);
    }

    // Writes to `GasSnapshotTest` group with `testSnapshotGas` name.
    function testSnapshotGasLastCallName() public {
        flare.run(1);

        uint256 gasUsed = vm.snapshotGasLastCall("testSnapshotGasLastCallName");
        assertGt(gasUsed, 0);
    }

    // Writes to `CustomGroup` group with `testSnapshotGas` name.
    function testSnapshotGasLastCallGroupName() public {
        flare.run(1);

        uint256 gasUsed = vm.snapshotGasLastCall("CustomGroup", "testSnapshotGasLastCallGroupName");
        assertGt(gasUsed, 0);
    }
}

contract GasSnapshotComparisonIsolatedTest is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    uint256 public slot0;
    uint256 public slot1;

    uint256 public cachedGas;

    function testGasComparisonEmpty() public {
        // Start a cheatcode snapshot.
        vm.startSnapshotGas("ComparisonGroup", "testGasComparisonEmptyA");
        uint256 a = vm.stopSnapshotGas();

        // Start a comparitive Solidity snapshot.
        _snapStart();
        uint256 b = _snapEnd();
        vm.snapshotValue("ComparisonGroup", "testGasComparisonEmptyB", b);

        // Start a comparitive Solidity snapshot.
        _snapStart();
        uint256 c = _snapEnd();
        vm.snapshotValue("ComparisonGroup", "testGasComparisonEmptyB", c);

        vm.assertEq(a, b);
        vm.assertEq(a, c);
    }

    function testGasComparisonInternalCold() public {
        // Start a cheatcode snapshot.
        vm.startSnapshotGas("ComparisonGroup", "testGasComparisonInternalColdA");
        slot0 = 1;
        uint256 a = vm.stopSnapshotGas();

        // Start a comparitive Solidity snapshot.
        _snapStart();
        slot1 = 1;
        uint256 b = _snapEnd();
        vm.snapshotValue("ComparisonGroup", "testGasComparisonInternalColdB", b);

        vm.assertApproxEqAbs(a, b, 11);
    }

    function testGasComparisonInternalWarm() public {
        // Warm up the cache.
        slot0 = 1;

        // Start a cheatcode snapshot.
        vm.startSnapshotGas("ComparisonGroup", "testGasComparisonInternalWarmA");
        slot0 = 2;
        uint256 a = vm.stopSnapshotGas();

        // Start a comparitive Solidity snapshot.
        _snapStart();
        slot0 = 3;
        uint256 b = _snapEnd();
        vm.snapshotValue("ComparisonGroup", "testGasComparisonInternalWarmB", b);

        vm.assertApproxEqAbs(a, b, 11);
    }

    function testGasComparisonExternal() public {
        // Warm up the cache.
        TargetB target = new TargetB();
        target.update(1);

        // Start a cheatcode snapshot.
        vm.startSnapshotGas("ComparisonGroup", "testGasComparisonExternalA");
        target.update(2);
        uint256 a = vm.stopSnapshotGas();

        // Start a comparitive Solidity snapshot.
        _snapStart();
        target.update(3);
        uint256 b = _snapEnd();
        vm.snapshotValue("ComparisonGroup", "testGasComparisonExternalB", b);
    }

    function testGasComparisonCreate() public {
        // Start a cheatcode snapshot.
        vm.startSnapshotGas("ComparisonGroup", "testGasComparisonCreateA");
        new TargetC();
        uint256 a = vm.stopSnapshotGas();

        // Start a comparitive Solidity snapshot.
        _snapStart();
        new TargetC();
        uint256 b = _snapEnd();
        vm.snapshotValue("ComparisonGroup", "testGasComparisonCreateB", b);

        vm.assertApproxEqAbs(a, b, 7);
    }

    function testGasComparisonNestedCalls() public {
        // Warm up the cache.
        TargetA target = new TargetA();
        target.update(1);

        // Start a cheatcode snapshot.
        vm.startSnapshotGas("ComparisonGroup", "testGasComparisonNestedCallsA");
        target.update(2);
        uint256 a = vm.stopSnapshotGas();

        // Start a comparitive Solidity snapshot.
        _snapStart();
        target.update(3);
        uint256 b = _snapEnd();
        vm.snapshotValue("ComparisonGroup", "testGasComparisonNestedCallsB", b);

        vm.assertApproxEqAbs(a, b, 7);
    }

    function testGasComparisonFlare() public {
        // Warm up the cache.
        Flare flare = new Flare();
        flare.run(1);

        // Start a cheatcode snapshot.
        vm.startSnapshotGas("ComparisonGroup", "testGasComparisonFlareA");
        flare.run(256);
        uint256 a = vm.stopSnapshotGas();

        // Start a comparitive Solidity snapshot.
        _snapStart();
        flare.run(256);
        uint256 b = _snapEnd();
        vm.snapshotValue("ComparisonGroup", "testGasComparisonFlareB", b);

        vm.assertApproxEqAbs(a, b, 7);
    }

    uint256 constant SNAPSHOT_OVERHEAD = 136;

    // Internal function to start a Solidity snapshot.
    function _snapStart() internal {
        cachedGas = 1;
        cachedGas = gasleft();
    }

    // Internal function to end a Solidity snapshot.
    function _snapEnd() internal returns (uint256 gasUsed) {
        gasUsed = cachedGas - gasleft() - SNAPSHOT_OVERHEAD;
        cachedGas = 2;
    }
}

contract Flare {
    bytes32[] public data;

    function run(uint256 n_) public {
        for (uint256 i = 0; i < n_; i++) {
            data.push(keccak256(abi.encodePacked(i)));
        }
    }
}

contract TargetA {
    TargetB public target;

    constructor() {
        target = new TargetB();
    }

    function update(uint256 x_) public {
        target.update(x_);
    }
}

contract TargetB {
    uint256 public x;

    function update(uint256 x_) public {
        x = x_;
    }
}

contract TargetC {}
