// SPDX-License-Identifier: MIT
pragma solidity >=0.5.12 <=0.8.0;

contract HelloWorld {
    string public message = "Hello_world!";

    function hello_world() public view returns (string memory) {
        return message;
    }
}