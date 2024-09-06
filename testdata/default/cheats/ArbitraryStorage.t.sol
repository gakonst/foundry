// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.18;

import "ds-test/test.sol";
import "cheats/Vm.sol";

contract Counter {
    uint256 public a;
    address public b;
    int8 public c;
    address[] public owners;

    function setA(uint256 _a) public {
        a = _a;
    }

    function setB(address _b) public {
        b = _b;
    }

    function getOwner(uint256 pos) public view returns (address) {
        return owners[pos];
    }

    function setOwner(uint256 pos, address owner) public {
        owners[pos] = owner;
    }
}

contract CounterArbitraryStorageWithSeedTest is DSTest {
    Vm vm = Vm(HEVM_ADDRESS);

    function test_fresh_storage() public {
        uint256 index = 55;
        Counter counter = new Counter();
        vm.setArbitraryStorage(address(counter));
        // Next call would fail with array out of bounds without arbitrary storage.
        address owner = counter.getOwner(index);
        // Subsequent calls should retrieve same value
        assertEq(counter.getOwner(index), owner);
        // Change slot and make sure new value retrieved
        counter.setOwner(index, address(111));
        assertEq(counter.getOwner(index), address(111));
    }

    function test_arbitrary_storage_warm() public {
        Counter counter = new Counter();
        vm.setArbitraryStorage(address(counter));
        assertGt(counter.a(), 0);
        counter.setA(0);
        // This should remain 0 if explicitly set.
        assertEq(counter.a(), 0);
        counter.setA(11);
        assertEq(counter.a(), 11);
    }

    function test_arbitrary_storage_multiple_read_writes() public {
        Counter counter = new Counter();
        vm.setArbitraryStorage(address(counter));
        uint256 slot1 = vm.randomUint(0, 100);
        uint256 slot2 = vm.randomUint(0, 100);
        require(slot1 != slot2, "random positions should be different");
        address alice = counter.owners(slot1);
        address bob = counter.owners(slot2);
        require(alice != bob, "random storage values should be different");
        counter.setOwner(slot1, bob);
        counter.setOwner(slot2, alice);
        assertEq(alice, counter.owners(slot2));
        assertEq(bob, counter.owners(slot1));
    }
}

contract AContract {
    uint256[] public a;
    address[] public b;
    int8[] public c;
    bytes32[] public d;
}

contract AContractArbitraryStorageTest is DSTest {
    Vm vm = Vm(HEVM_ADDRESS);

    function test_arbitrary_storage_with_seed() public {
        AContract target = new AContract();
        vm.setArbitraryStorage(address(target));
        assertEq(target.a(11), 85286582241781868037363115933978803127245343755841464083427462398552335014708);
        assertEq(target.b(22), 0x939180Daa938F9e18Ff0E76c112D25107D358B02);
        assertEq(target.c(33), -104);
        assertEq(target.d(44), 0x6c178fa9c434f142df61a5355cc2b8d07be691b98dabf5b1a924f2bce97a19c7);
    }
}

contract SymbolicStore {
    uint256 public testNumber = 1337; // slot 0

    constructor() {}
}

contract SymbolicStorageTest is DSTest {
    Vm vm = Vm(HEVM_ADDRESS);

    function test_SymbolicStorage() public {
        uint256 slot = vm.randomUint(0, 100);
        address addr = 0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8;
        vm.setArbitraryStorage(addr);
        bytes32 value = vm.load(addr, bytes32(slot));
        assertEq(uint256(value), 85286582241781868037363115933978803127245343755841464083427462398552335014708);
        // Load slot again and make sure we get same value.
        bytes32 value1 = vm.load(addr, bytes32(slot));
        assertEq(uint256(value), uint256(value1));
    }

    function test_SymbolicStorage1() public {
        uint256 slot = vm.randomUint(0, 100);
        SymbolicStore myStore = new SymbolicStore();
        vm.setArbitraryStorage(address(myStore));
        bytes32 value = vm.load(address(myStore), bytes32(uint256(slot)));
        assertEq(uint256(value), 85286582241781868037363115933978803127245343755841464083427462398552335014708);
    }

    function testEmptyInitialStorage(uint256 slot) public {
        bytes32 storage_value = vm.load(address(vm), bytes32(slot));
        assertEq(uint256(storage_value), 0);
    }
}
