// SPDX-License-Identifier: Unlicense
pragma solidity >=0.8.0;

import "ds-test/test.sol";
import "./Cheats.sol";

contract ParseTest is DSTest {
    Cheats constant cheats = Cheats(HEVM_ADDRESS);

    function testParseBytes() public {
        bytes memory testBytes = hex"7109709ECfa91a80626fF3989D68f67F5b1DD12D";

        string memory stringBytes = "0x7109709ECfa91a80626fF3989D68f67F5b1DD12D";
        assertEq(testBytes, cheats.parseBytes(stringBytes));

        stringBytes = "7109709ECfa91a80626fF3989D68f67F5b1DD12D";
        assertEq(testBytes, cheats.parseBytes(stringBytes));
    }

    function testParseBytesFuzzed(bytes memory testBytes) public {
        string memory stringBytes = cheats.toString(testBytes);
        assertEq(testBytes, cheats.parseBytes(stringBytes));
    }

    function testParseAddress() public {
        address testAddress = 0x7109709ECfa91a80626fF3989D68f67F5b1DD12D;

        string memory stringAddress = "0x7109709ECfa91a80626fF3989D68f67F5b1DD12D";
        assertEq(testAddress, cheats.parseAddress(stringAddress));

        stringAddress = "7109709ECfa91a80626fF3989D68f67F5b1DD12D";
        assertEq(testAddress, cheats.parseAddress(stringAddress));
    }

    function testParseAddressFuzzed(address testAddress) public {
        string memory stringAddress = cheats.toString(testAddress);
        assertEq(testAddress, cheats.parseAddress(stringAddress));
    }

    function testParseUint() public {
        uint256 testUint = 420;

        string memory stringUint = "420";
        assertEq(testUint, cheats.parseUint(stringUint));
    }

    function testParseUintFuzzed(uint256 testUint) public {
        string memory stringUint = cheats.toString(testUint);
        assertEq(testUint, cheats.parseUint(stringUint));
    }

    function testParseInt() public {
        int256 testInt = 420;

        string memory stringInt = "420";
        assertEq(testInt, cheats.parseInt(stringInt));
    }

    function testParseIntFuzzed(int256 testInt) public {
        string memory stringInt = cheats.toString(testInt);
        assertEq(testInt, cheats.parseInt(stringInt));
    }

    function testParseBytes32() public {
        bytes32 testBytes = "test";

        string memory stringBytes = "7465737400000000000000000000000000000000000000000000000000000000";
        assertEq(testBytes, cheats.parseBytes32(stringBytes));

        stringBytes = "0x7465737400000000000000000000000000000000000000000000000000000000";
        assertEq(testBytes, cheats.parseBytes32(stringBytes));
    }

    function testParseBytes32Fuzzed(bytes32 testBytes) public {
        string memory stringBytes = cheats.toString(testBytes);
        assertEq(testBytes, cheats.parseBytes32(stringBytes));
    }

    function testParseBool() public {
        bool testBool = true;

        string memory stringBool = "true";
        assertEq(testBool, cheats.parseBool(stringBool));
    }

    function testParseBoolFuzzed(bool testBool) public {
        string memory stringBool = cheats.toString(testBool);
        assertEq(testBool, cheats.parseBool(stringBool));
    }
}
