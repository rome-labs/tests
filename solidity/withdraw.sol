// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IPredeployed {
    function withdrawal(bytes32 target) external;
}

contract Caller {
    address constant PREDEPLOYED_ADDRESS = 0x4200000000000000000000000000000000000016;

    function call1SOLWithdrawal(bytes32 withdrawalTarget) external payable {
        bytes memory callData = abi.encodeWithSelector(
            IPredeployed.withdrawal.selector,
            withdrawalTarget
        );

        // Hardcode the value to 1 SOL (1 * 10^18 wei) instead of msg.value
        uint256 hardcodedValue = 1_000_000_000_000_000_000;

        (bool success, ) = PREDEPLOYED_ADDRESS.call{value: hardcodedValue}(callData);
        require(success, "withdrawal failed");
    }

    function callAnySOLWithdrawal(bytes32 withdrawalTarget) external payable {
        bytes memory callData = abi.encodeWithSelector(
            IPredeployed.withdrawal.selector,
            withdrawalTarget
        );

        (bool success, ) = PREDEPLOYED_ADDRESS.call{value: msg.value}(callData);
        require(success, "withdrawal failed");
    }

    function getPredeployedBalance() public view returns (uint256) {
        return PREDEPLOYED_ADDRESS.balance;
    }

    function getBalance() public view returns (uint256) {
        return address(this).balance;
    }

    receive() external payable {}
}
