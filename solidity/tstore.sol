// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

contract TStore {
    function set(uint val) private {
        assembly {
            tstore(0, val)
        }
    }

    function get() private view returns (uint val) {
        assembly {
            val := tload(0)
        }
    }

    function check(uint value) public {
        set(value);
        uint load = get();

        require(load == value);
    }
}