// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.18;

import "ds-test/test.sol";
import "cheats/Vm.sol";

contract ArbitraryCheatcodesTest is DSTest {
    Vm vm = Vm(HEVM_ADDRESS);

    int128 constant min = -170141183460469231731687303715884105728;
    int128 constant max = 170141183460469231731687303715884105727;

    function test_int128() public {
        int256 val = vm.arbitraryInt(16);
        assertGe(val, min);
        assertLe(val, max);
    }

    function testFail_int128() public {
        int256 val = vm.arbitraryInt(16);
        assertGt(val, max);
    }

    function test_address() public {
        address fresh_address = vm.arbitraryAddress();
        assert(fresh_address != address(this));
        assert(fresh_address != address(vm));
    }

    function test_arbitraryUints(uint8 x) public {
        vm.assume(0 < x);
        vm.assume(x <= 32);
        uint256 freshUint = vm.arbitraryUint(x);

        assert(0 <= freshUint);
        if (x == 32) {
            assert(freshUint <= type(uint256).max);
        } else {
            assert(freshUint <= 2 ** (8 * x) - 1);
        }
    }

    function test_arbitrarySymbolicWord() public {
        uint256 freshUint192 = vm.arbitraryUint(192);

        assert(0 <= freshUint192);
        assert(freshUint192 <= type(uint192).max);
    }
}

contract ArbitraryBytesTest is DSTest {
    Vm vm = Vm(HEVM_ADDRESS);

    bytes1 local_byte;
    bytes local_bytes;

    uint256 constant length_limit = 72;

    function manip_symbolic_bytes(bytes memory b) public {
        uint256 middle = b.length / 2;
        b[middle] = hex"aa";
    }

    function test_symbolic_bytes_1() public {
        uint256 length = uint256(vm.arbitraryUint(1, type(uint8).max));
        bytes memory fresh_bytes = vm.arbitraryBytes(length);
        uint256 index = uint256(vm.arbitraryUint(1));

        local_byte = fresh_bytes[index];
        assertEq(fresh_bytes[index], local_byte);
    }

    function test_symbolic_bytes_2() public {
        uint256 length = uint256(vm.arbitraryUint(1, type(uint8).max));
        bytes memory fresh_bytes = vm.arbitraryBytes(length);

        local_bytes = fresh_bytes;
        assertEq(fresh_bytes, local_bytes);
    }

    function test_symbolic_bytes_3() public {
        uint256 length = uint256(vm.arbitraryUint(1, type(uint8).max));
        bytes memory fresh_bytes = vm.arbitraryBytes(length);

        manip_symbolic_bytes(fresh_bytes);
        assertEq(hex"aa", fresh_bytes[length / 2]);
    }

    function test_symbolic_bytes_length(uint8 l) public {
        vm.assume(0 < l);
        bytes memory fresh_bytes = vm.arbitraryBytes(l);
        assertEq(fresh_bytes.length, l);
    }
}
