// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

contract  Destruct{
    uint public val = 1;
    constructor() payable {}

    function get() public view returns(uint) {
        return val;
    }
    function destruct() public {
        address payable sender = payable (address(msg.sender));
        selfdestruct(sender);
    }
}

contract DestructCaller {
    address payable  addr1;
    address payable  addr2;

    constructor() payable {}

    function deploy() public payable {
        Destruct a = new Destruct{value: 0}();
        addr1 = payable (address(a));
        require(a.get() == 1);
    }

    function deploy_and_destruct() public payable {
        Destruct a = new Destruct{value: 0}();
        addr2 = payable (address(a));
        require(a.get() == 1);
        a.destruct();
    }

    function check() public  {
        Destruct a1 = Destruct(addr1);
        require(a1.get() == 1);

        (bool success1, bytes memory data1) = addr1.call(abi.encodeWithSignature("get()"));
        require (success1 == true);
        require(data1.length == 32);

        // contract has not been deployed
        (bool success2, bytes memory data2) = addr2.call(abi.encodeWithSignature("get()"));
        require (success2 == true);
        require(data2.length == 0);
   }
}

