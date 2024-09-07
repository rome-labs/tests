// SPDX-License-Identifier: MIT
pragma solidity >=0.5.12 <=0.8.0;

contract HelloWorld {
//    string public message = "Hello world!";
    uint public num = 7;

    function get() public view returns (uint) {
        return num;
    }
//    function hello_world() public view returns (string memory) {
//        return message;
//    }
}