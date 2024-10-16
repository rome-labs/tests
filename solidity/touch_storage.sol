// SPDX-License-Identifier: MIT
pragma solidity >=0.5.12 <=0.8.0;

import "./hello_world.sol";

contract TouchStorage {
    uint public value = 0;
    uint[] vec;
    string public text;
    string public hello_world;

    function set_value(uint a) public {
        value = a;
    }
    function get_value() public view returns(uint) {
        return value;
    }
    function push_vec(uint a) public {
        vec.push(a);
    }
    function get_vec(uint i) public view returns(uint) {
        return vec[i];
    }
    function set_text(string memory new_text) public {
        text = new_text;
    }
    function get_text() public view returns (string memory) {
        return text;
    }
    function get_local() public pure returns(uint) {
        uint x = 5;
        return x;
    }
    function deploy() public returns(address){
        HelloWorld hello = new HelloWorld();
        hello_world = hello.hello_world();
        return address(hello);
    }
}
