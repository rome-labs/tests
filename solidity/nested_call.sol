// SPDX-License-Identifier: MIT
pragma solidity >=0.5.12 <=0.8.0;

import "./touch_storage.sol";

contract NestedCall {
    TouchStorage nested;
    string public text;

    constructor(address addr) {
        nested = TouchStorage(addr);
    }
    function set_value(uint a) public {
        nested.set_value(a);
    }
    function get_value() public view returns(uint) {
        return nested.get_value();
    }
    function push_vec(uint a) public {
        nested.push_vec(a);
    }
    function get_vec(uint i) public view returns(uint) {
        return nested.get_vec(i);
    }
    function set_text(string memory new_text) public {
        nested.set_text(new_text);
    }
    function get_text() public view returns (string memory) {
        return nested.get_text();
    }
    function get_local() public view returns(uint) {
        return nested.get_local();
    }
    function deploy() public {
         address addr =  nested.deploy();
         HelloWorld hello = HelloWorld(addr);
         text = hello.hello_world();
    }
}

