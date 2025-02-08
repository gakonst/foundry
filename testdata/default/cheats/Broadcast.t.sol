// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.18;

import "ds-test/test.sol";
import "cheats/Vm.sol";

library F {
    function t2() public pure returns (uint256) {
        return 1;
    }
}

contract Test is DSTest {
    using F for *;

    uint256 public changed = 0;

    function t(uint256 a) public returns (uint256) {
        uint256 b = 0;
        for (uint256 i; i < a; i++) {
            b += F.t2();
        }
        emit log_string("here");
        return b;
    }

    function inc() public returns (uint256) {
        changed += 1;
    }

    function multiple_arguments(uint256 a, address b, uint256[] memory c) public returns (uint256) {}

    function echoSender() public view returns (address) {
        return msg.sender;
    }
}

contract BroadcastTest is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    // 1st anvil account
    address public ACCOUNT_A = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;
    // 2nd anvil account
    address public ACCOUNT_B = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8;

    function deploy() public {
        vm.broadcast(ACCOUNT_A);
        Test test = new Test();

        // this wont generate tx to sign
        uint256 b = test.t(4);

        // this will
        vm.broadcast(ACCOUNT_B);
        test.t(2);
    }

    function deployPrivateKey() public {
        string memory mnemonic = "test test test test test test test test test test test junk";

        uint256 privateKey = vm.deriveKey(mnemonic, 3);
        assertEq(privateKey, 0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6);

        vm.broadcast(privateKey);
        Test test = new Test();

        vm.startBroadcast(privateKey);
        Test test2 = new Test();
        vm.stopBroadcast();
    }

    function deployRememberKey() public {
        string memory mnemonic = "test test test test test test test test test test test junk";

        uint256 privateKey = vm.deriveKey(mnemonic, 3);
        assertEq(privateKey, 0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6);

        address thisAddress = vm.rememberKey(privateKey);
        assertEq(thisAddress, 0x90F79bf6EB2c4f870365E785982E1f101E93b906);

        vm.broadcast(thisAddress);
        Test test = new Test();
    }

    function deployRememberKeyResume() public {
        vm.broadcast(ACCOUNT_A);
        Test test = new Test();

        string memory mnemonic = "test test test test test test test test test test test junk";

        uint256 privateKey = vm.deriveKey(mnemonic, 3);
        address thisAddress = vm.rememberKey(privateKey);

        vm.broadcast(thisAddress);
        Test test2 = new Test();
    }

    function deployOther() public {
        vm.startBroadcast(ACCOUNT_A);
        Test tmptest = new Test();
        Test test = new Test();

        // won't trigger a transaction: staticcall
        test.changed();

        // won't trigger a transaction: staticcall
        require(test.echoSender() == ACCOUNT_A);

        // will trigger a transaction
        test.t(1);

        // will trigger a transaction
        test.inc();

        vm.stopBroadcast();

        require(test.echoSender() == address(this));

        vm.broadcast(ACCOUNT_B);
        Test tmptest2 = new Test();

        vm.broadcast(ACCOUNT_B);
        // will trigger a transaction
        test.t(2);

        vm.broadcast(ACCOUNT_B);
        // will trigger a transaction from B
        payable(ACCOUNT_A).transfer(2);

        vm.broadcast(ACCOUNT_B);
        // will trigger a transaction
        test.inc();

        assert(test.changed() == 2);
    }

    function deployPanics() public {
        vm.broadcast(address(0x1337));
        Test test = new Test();

        // This panics because this would cause an additional relinking that isnt conceptually correct
        // from a solidity standpoint. Basically, this contract `BroadcastTest`, injects the code of
        // `Test` *into* its code. So it isn't reasonable to break solidity to our will of having *two*
        // versions of `Test` based on the sender/linker.
        vm.broadcast(address(0x1338));
        new Test();

        vm.broadcast(address(0x1338));
        test.t(0);
    }

    function deployNoArgs() public {
        vm.startBroadcast();
        Test test1 = new Test();

        Test test2 = new Test();
        vm.stopBroadcast();
    }

    /// forge-config: default.allow_internal_expect_revert = true
    // function testRevertIfNoBroadcast() public {
    //     vm.expectRevert();
    //     vm.stopBroadcast();
    // }
}

contract NoLink is DSTest {
    function t(uint256 a) public returns (uint256) {
        uint256 b = 0;
        for (uint256 i; i < a; i++) {
            b += i;
        }
        emit log_string("here");
        return b;
    }

    function view_me() public pure returns (uint256) {
        return 1337;
    }
}

interface INoLink {
    function t(uint256 a) external returns (uint256);
}

contract BroadcastTestNoLinking is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    // ganache-cli -d 1st
    address public ACCOUNT_A = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;

    // ganache-cli -d 2nd
    address public ACCOUNT_B = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8;

    function deployDoesntPanic() public {
        vm.broadcast(address(ACCOUNT_A));
        NoLink test = new NoLink();

        vm.broadcast(address(ACCOUNT_B));
        new NoLink();

        vm.broadcast(address(ACCOUNT_B));
        test.t(0);
    }

    function deployMany() public {
        assert(vm.getNonce(msg.sender) == 0);

        vm.startBroadcast();

        for (uint256 i; i < 25; i++) {
            NoLink test9 = new NoLink();
        }

        vm.stopBroadcast();
    }

    function deployCreate2() public {
        vm.startBroadcast();
        NoLink test_c2 = new NoLink{salt: bytes32(uint256(1337))}();
        assert(test_c2.view_me() == 1337);
        NoLink test2 = new NoLink();
        vm.stopBroadcast();
    }

    function deployCreate2(address deployer) public {
        vm.startBroadcast();
        bytes32 salt = bytes32(uint256(1338));
        NoLink test_c2 = new NoLink{salt: salt}();
        assert(test_c2.view_me() == 1337);

        address expectedAddress = address(
            uint160(
                uint256(
                    keccak256(
                        abi.encodePacked(
                            bytes1(0xff),
                            deployer,
                            salt,
                            keccak256(abi.encodePacked(type(NoLink).creationCode, abi.encode()))
                        )
                    )
                )
            )
        );
        require(address(test_c2) == expectedAddress, "Create2 address mismatch");

        NoLink test2 = new NoLink();
        vm.stopBroadcast();
    }

    function errorStaticCall() public {
        vm.broadcast();
        NoLink test11 = new NoLink();

        vm.broadcast();
        test11.view_me();
    }
}

contract BroadcastMix is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    // ganache-cli -d 1st
    address public ACCOUNT_A = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;

    // ganache-cli -d 2nd
    address public ACCOUNT_B = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8;

    function more() internal {
        vm.broadcast();
        NoLink test11 = new NoLink();
    }

    function deployMix() public {
        address user = msg.sender;
        assert(user == address(0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266));

        NoLink no = new NoLink();

        vm.startBroadcast();
        NoLink test1 = new NoLink();
        test1.t(2);
        NoLink test2 = new NoLink();
        test2.t(2);
        vm.stopBroadcast();

        vm.startBroadcast(user);
        NoLink test3 = new NoLink();
        NoLink test4 = new NoLink();
        test4.t(2);
        vm.stopBroadcast();

        vm.broadcast();
        test4.t(2);

        vm.broadcast();
        NoLink test5 = new NoLink();

        vm.broadcast();
        INoLink test6 = INoLink(address(new NoLink()));

        vm.broadcast();
        NoLink test7 = new NoLink();

        vm.broadcast(user);
        NoLink test8 = new NoLink();

        vm.broadcast();
        NoLink test9 = new NoLink();

        vm.broadcast(user);
        NoLink test10 = new NoLink();

        more();
    }
}

contract BroadcastTestSetup is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    function setUp() public {
        // It predeployed a library first
        assert(vm.getNonce(msg.sender) == 1);

        vm.broadcast();
        Test t = new Test();

        vm.broadcast();
        t.t(2);
    }

    function run() public {
        vm.broadcast();
        new NoLink();

        vm.broadcast();
        Test t = new Test();

        vm.broadcast();
        t.t(3);
    }
}

contract BroadcastTestLog is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    function run() public {
        uint256[] memory arr = new uint256[](2);
        arr[0] = 3;
        arr[1] = 4;

        vm.startBroadcast();
        {
            Test c1 = new Test();
            Test c2 = new Test{salt: bytes32(uint256(1337))}();

            c1.multiple_arguments(1, address(0x1337), arr);
            c1.inc();
            c2.t(1);

            payable(address(0x1337)).transfer(0.0001 ether);
        }
        vm.stopBroadcast();
    }
}

contract TestInitialBalance is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    function runCustomSender() public {
        // Make sure we're testing a different caller than the default one.
        assert(msg.sender != address(0x00a329c0648769A73afAc7F9381E08FB43dBEA72));

        // NodeConfig::test() sets the balance of the address used in this test to 100 ether.
        assert(msg.sender.balance == 100 ether);

        vm.broadcast();
        new NoLink();
    }

    function runDefaultSender() public {
        // Make sure we're testing with the default caller.
        assert(msg.sender == address(0x1804c8AB1F12E6bbf3894d4083f33e07309d1f38));

        assert(msg.sender.balance == type(uint256).max);

        vm.broadcast();
        new NoLink();
    }
}

contract MultiChainBroadcastNoLink is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    // ganache-cli -d 1st
    address public ACCOUNT_A = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;

    // ganache-cli -d 2nd
    address public ACCOUNT_B = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8;

    function deploy(string memory sforkA, string memory sforkB) public {
        uint256 forkA = vm.createFork(sforkA);
        uint256 forkB = vm.createFork(sforkB);

        vm.selectFork(forkA);
        vm.broadcast(address(ACCOUNT_A));
        new NoLink();
        vm.broadcast(address(ACCOUNT_B));
        new NoLink();
        vm.selectFork(forkB);
        vm.startBroadcast(address(ACCOUNT_B));
        new NoLink();
        new NoLink();
        new NoLink();
        vm.stopBroadcast();
        vm.startBroadcast(address(ACCOUNT_A));
        new NoLink();
        new NoLink();
    }

    function deployError(string memory sforkA, string memory sforkB) public {
        uint256 forkA = vm.createFork(sforkA);
        uint256 forkB = vm.createFork(sforkB);

        vm.selectFork(forkA);
        vm.broadcast(address(ACCOUNT_A));
        new NoLink();
        vm.startBroadcast(address(ACCOUNT_B));
        new NoLink();

        vm.selectFork(forkB);
        vm.broadcast(address(ACCOUNT_B));
        new NoLink();
    }
}

contract MultiChainBroadcastLink is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    // ganache-cli -d 1st
    address public ACCOUNT_A = 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266;

    // ganache-cli -d 2nd
    address public ACCOUNT_B = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8;

    function deploy(string memory sforkA, string memory sforkB) public {
        uint256 forkA = vm.createFork(sforkA);
        uint256 forkB = vm.createFork(sforkB);

        vm.selectFork(forkA);
        vm.broadcast(address(ACCOUNT_B));
        new Test();

        vm.selectFork(forkB);
        vm.broadcast(address(ACCOUNT_B));
        new Test();
    }
}

contract BroadcastEmptySetUp is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    function setUp() public {}

    function run() public {
        vm.broadcast();
        new Test();
    }
}

contract ContractA {
    uint256 var1;

    constructor(address script_caller) {
        require(msg.sender == script_caller);
        require(tx.origin == script_caller);
    }

    function method(address script_caller) public {
        require(msg.sender == script_caller);
        require(tx.origin == script_caller);
    }
}

contract ContractB {
    uint256 var2;

    constructor(address script_caller) {
        require(address(0x1337) != script_caller);
        require(msg.sender == address(0x1337));
        require(tx.origin == address(0x1337));
    }

    function method(address script_caller) public {
        require(address(0x1337) != script_caller);
        require(msg.sender == address(0x1337));
        require(tx.origin == address(0x1337));
    }
}

contract CheckOverrides is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    function run() external {
        // `script_caller` can be set by `--private-key ...` or `--sender ...`
        // Otherwise it will take the default value of 0x1804c8AB1F12E6bbf3894d4083f33e07309d1f38
        address script_caller = msg.sender;
        require(script_caller == 0x1804c8AB1F12E6bbf3894d4083f33e07309d1f38);
        require(tx.origin == script_caller);

        // startBroadcast(script_caller)
        vm.startBroadcast();
        require(tx.origin == script_caller);
        require(msg.sender == script_caller);

        ContractA a = new ContractA(script_caller);
        require(tx.origin == script_caller);
        require(msg.sender == script_caller);

        a.method(script_caller);
        require(tx.origin == script_caller);
        require(msg.sender == script_caller);

        vm.stopBroadcast();

        // startBroadcast(msg.sender)
        vm.startBroadcast(address(0x1337));
        require(tx.origin == script_caller);
        require(msg.sender == script_caller);
        require(msg.sender != address(0x1337));

        ContractB b = new ContractB(script_caller);
        require(tx.origin == script_caller);
        require(msg.sender == script_caller);

        b.method(script_caller);
        require(tx.origin == script_caller);
        require(msg.sender == script_caller);

        vm.stopBroadcast();
    }
}

contract Child {}

contract Parent {
    constructor() {
        new Child();
    }
}

contract ScriptAdditionalContracts is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);

    function run() external {
        vm.startBroadcast();
        new Parent();
    }
}

contract SignatureTester {
    address public immutable owner;

    constructor() {
        owner = msg.sender;
    }

    function verifySignature(bytes32 digest, uint8 v, bytes32 r, bytes32 s) public view returns (bool) {
        require(ecrecover(digest, v, r, s) == owner, "Invalid signature");
    }
}

contract ScriptSign is DSTest {
    Vm constant vm = Vm(HEVM_ADDRESS);
    bytes32 digest = keccak256("something");

    function run() external {
        vm.startBroadcast();
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(digest);

        vm._expectCheatcodeRevert(
            bytes(string.concat("signer with address ", vm.toString(address(this)), " is not available"))
        );
        vm.sign(address(this), digest);

        SignatureTester tester = new SignatureTester();
        (, address caller,) = vm.readCallers();
        assertEq(tester.owner(), caller);
        tester.verifySignature(digest, v, r, s);
    }

    function run(address sender) external {
        vm._expectCheatcodeRevert(bytes("could not determine signer"));
        vm.sign(digest);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(sender, digest);
        address actual = ecrecover(digest, v, r, s);

        assertEq(actual, sender);
    }
}
