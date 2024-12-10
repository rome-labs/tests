// SPDX-License-Identifier: MIT
pragma solidity <=0.8.28;

contract CU {
    uint[] new_vec;
    uint[] vec = new uint[](150);

    function update() public {
        uint i = 0;

        while(i < 150) {
            vec[i] = i;
            i = i + 1;
        }
    }

    function push() public {
        uint i = 0;

        while(i < 100) {
            new_vec.push(i);
            i = i + 1;
        }
    }

    function update_single() public {
        uint i = 0;

        while(i < 150) {
            vec[0] = i;
            i = i + 1;
        }
    }
}