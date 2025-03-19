// SPDX-License-Identifier: MIT
pragma solidity =0.8.28;

contract TestTransientStorage {
    function store(uint256 value) private {
        assembly {
            tstore(1, value)
        }
    }

    function load() private view returns (uint256 value) {
        assembly {
            value := tload(1)
        }
    }

    function callTransientStorage(uint256 value) public {
        store(value); 
        uint256 loadedValue = load();

        require(loadedValue == value);
    }
}
