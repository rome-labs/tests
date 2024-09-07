// SPDX-License-Identifier: MIT
pragma solidity >=0.5.12 <=0.8.0;

import "./storage.sol";

contract StorageCaller {
    Storage st;

    constructor(address storage_address) {
        st = Storage(storage_address);
    }
    function change() public {
        st.change();
    }
    function get() public view returns(uint) {
        return st.get();
    }
    function get_local() public view returns(uint) {
        return st.get_local();
    }
    function add() public {
        st.add();
    }
    function get_text() public view returns (string memory) {
        return st.get_text();
    }
    function update_text(string memory new_text) public {
        st.update_text(new_text);
    }
}

