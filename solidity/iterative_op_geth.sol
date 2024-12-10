// SPDX-License-Identifier: MIT
pragma solidity <=0.8.28;

contract IterativeOpGeth {
    uint value = 0;

    function atomic()  public {
        uint i = 0;

        while (i < 3) {
            value = value + 1;
            i = i + 1;
        }
    }

    function iterative()  public {
        uint i = 0;

        while (i < 100) {
            value = value + 1;
            i = i + 1;
        }
    }
}
