// SPDX-License-Identifier: MIT
pragma solidity <=0.8.0;

contract UpdateStorage {
    uint a = 0;
    function update ()  public {

        while (a < 10) {
            a = a + 1;
        }
    }
}