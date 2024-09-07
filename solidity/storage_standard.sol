// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.5.12 <=0.8.0;

contract StorageStandard {

    uint256 number;

    /**
     * @dev Store value in variable
     * @param num value to store
     */
    function store(uint256 num) public {
        number = num;
    }

    /**
     * @dev Return value
     * @return value of 'number'
     */
    function retrieve() public view returns (uint256){
        return number;
    }
}