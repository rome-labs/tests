// SPDX-License-Identifier: MIT
pragma solidity >=0.5.12 <=0.8.28;

contract GetStorageAt {
    uint public num = 7;

    function get() public view returns (uint) {
        return num;
    }
}