// SPDX-License-Identifier: MIT
pragma solidity >=0.5.12 <=0.8.0;

import "./hello_world.sol";

contract Storage {
    uint public value = 0;
    uint[] a;
    string public text;

    function change() public {
        value = value + 1;
    }
    function get() public view returns(uint) {
        return value;
    }
    function get_local() public view returns(uint) {
        uint x = 5;
        return x;
    }
    function add() public {
        a.push(value);
    }
    function get_text() public view returns (string memory) {
        return text;
    }
    function update_text(string memory new_text) public {
        text = new_text;
    }
    function deploy() public returns(address){
        HelloWorld hello = new HelloWorld();
        hello.get();
        return address(hello);
    }
}
