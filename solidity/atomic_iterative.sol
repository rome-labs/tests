// SPDX-License-Identifier: MIT
pragma solidity <=0.8.0;

contract AtomicIterative {
    uint value = 0;

    function atomic_rw()  public {
        uint i = 0;

        while (i < 3) {
            value = value + 1;
            i = i + 1;
        }
    }

    function atomic_ro() view  public {
        uint i = 0;
        uint local = 0;

        while (i < 3) {
            local = value;
            i = i + 1;
        }
    }

    function iterative_rw()  public {
        uint i = 0;

        while (i < 200) {
            value = value + 1;
            i = i + 1;
        }
    }

    function iterative_ro() view public {
        uint i = 0;
        uint local = 0;

        while (i < 200) {
            local = value;
            i = i + 1;
        }
    }
}
