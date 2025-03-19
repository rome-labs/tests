// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract B {
    uint public value;

    constructor() {
        value = 0;
    }

    function update_slot() public {
        value++;
    }

    function revert_() public pure {
        require(false, "Reverted");
    }

    function get_value() public view returns (uint) {
        return value;
    }
}

contract A {
    uint public value;
    B public contractB;

    function deploy_B() public {
        contractB = new B();
    }

    function update() public {
        value = value + 1;
    }

    function call_update_slot() public {
        require(address(contractB) != address(0), "B not deployed");
        contractB.update_slot();
    }

    function call_revert() public view {
        require(address(contractB) != address(0), "B not deployed");
        contractB.revert_();
    }

    function get_B_address() public view returns (address) {
        return address(contractB);
    }
}
