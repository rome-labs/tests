// SPDX-License-Identifier: MIT
pragma solidity <=0.8.28;

contract MultiStorage {
    uint256 public num1;  // Slot 0
    uint256 public num2;  // Slot 1
    bool public flag;     // Slot 2 (fits into a single slot)
    
    uint256[] public dynamicArray;  // Slot 3 (length stored at 3, data at keccak256(3))
    mapping(address => uint256) public balanceOf;  // Stored at keccak256(slot . key)

    struct User {
        uint256 id;
        bool active;
    }
    User public user;  // Stored starting at Slot 4

    function update() public {
        num1 = 12;
        num2 = 110;
        flag = true;
        dynamicArray.push(989);
        balanceOf[msg.sender] = 1434;
        user = User(11, true);
    }
}
