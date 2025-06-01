// Sources flattened with hardhat v2.23.0 https://hardhat.org

// SPDX-License-Identifier: MIT

// File @openzeppelin/contracts/utils/Context.sol@v5.3.0

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.0.1) (utils/Context.sol)

pragma solidity ^0.8.20;

/**
 * @dev Provides information about the current execution context, including the
 * sender of the transaction and its data. While these are generally available
 * via msg.sender and msg.data, they should not be accessed in such a direct
 * manner, since when dealing with meta-transactions the account sending and
 * paying for execution may not be the actual sender (as far as an application
 * is concerned).
 *
 * This contract is only required for intermediate, library-like contracts.
 */
abstract contract Context {
    function _msgSender() internal view virtual returns (address) {
        return msg.sender;
    }

    function _msgData() internal view virtual returns (bytes calldata) {
        return msg.data;
    }

    function _contextSuffixLength() internal view virtual returns (uint256) {
        return 0;
    }
}


// File @openzeppelin/contracts/access/Ownable.sol@v5.3.0

// Original license: SPDX_License_Identifier: MIT
// OpenZeppelin Contracts (last updated v5.0.0) (access/Ownable.sol)

pragma solidity ^0.8.20;

/**
 * @dev Contract module which provides a basic access control mechanism, where
 * there is an account (an owner) that can be granted exclusive access to
 * specific functions.
 *
 * The initial owner is set to the address provided by the deployer. This can
 * later be changed with {transferOwnership}.
 *
 * This module is used through inheritance. It will make available the modifier
 * `onlyOwner`, which can be applied to your functions to restrict their use to
 * the owner.
 */
abstract contract Ownable is Context {
    address private _owner;

    /**
     * @dev The caller account is not authorized to perform an operation.
     */
    error OwnableUnauthorizedAccount(address account);

    /**
     * @dev The owner is not a valid owner account. (eg. `address(0)`)
     */
    error OwnableInvalidOwner(address owner);

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    /**
     * @dev Initializes the contract setting the address provided by the deployer as the initial owner.
     */
    constructor(address initialOwner) {
        if (initialOwner == address(0)) {
            revert OwnableInvalidOwner(address(0));
        }
        _transferOwnership(initialOwner);
    }

    /**
     * @dev Throws if called by any account other than the owner.
     */
    modifier onlyOwner() {
        _checkOwner();
        _;
    }

    /**
     * @dev Returns the address of the current owner.
     */
    function owner() public view virtual returns (address) {
        return _owner;
    }

    /**
     * @dev Throws if the sender is not the owner.
     */
    function _checkOwner() internal view virtual {
        if (owner() != _msgSender()) {
            revert OwnableUnauthorizedAccount(_msgSender());
        }
    }

    /**
     * @dev Leaves the contract without owner. It will not be possible to call
     * `onlyOwner` functions. Can only be called by the current owner.
     *
     * NOTE: Renouncing ownership will leave the contract without an owner,
     * thereby disabling any functionality that is only available to the owner.
     */
    function renounceOwnership() public virtual onlyOwner {
        _transferOwnership(address(0));
    }

    /**
     * @dev Transfers ownership of the contract to a new account (`newOwner`).
     * Can only be called by the current owner.
     */
    function transferOwnership(address newOwner) public virtual onlyOwner {
        if (newOwner == address(0)) {
            revert OwnableInvalidOwner(address(0));
        }
        _transferOwnership(newOwner);
    }

    /**
     * @dev Transfers ownership of the contract to a new account (`newOwner`).
     * Internal function without access restriction.
     */
    function _transferOwnership(address newOwner) internal virtual {
        address oldOwner = _owner;
        _owner = newOwner;
        emit OwnershipTransferred(oldOwner, newOwner);
    }
}


// File contracts/interface.sol

// Original license: SPDX_License_Identifier: MIT
pragma solidity ^0.8.20;

interface ISplToken {
    struct Account {
        bytes32 mint;
        bytes32 owner;
        uint64 amount;
        bytes32 delegate;
        AccountState state;
        bool is_native;
        uint64 native_value;
        uint64 delegated_amount;
        bytes32 close_authority;
    }
    
    enum AccountState {
        Uninitialized,
        Initialized,
        Frozen
    }

    struct Seed{
        bytes item;
    }

    function transfer(bytes32 from, bytes32 to, uint64 amount, Seed[] memory seeds) external returns(bytes32);
    function initialize_account3(bytes32 acc, bytes32 mint, bytes32 owner) external returns(bytes32);
    function program_id() external view returns(bytes32);
    function account_state(bytes32) external view returns(Account memory);
}

interface IAssociatedSplToken {
    function create_associated_token_account(bytes32 wallet, bytes32 mint) external returns(bytes32);
    function program_id() external view returns(bytes32);
}

interface ISystemProgram {
    struct Seed{
        bytes item;
    }
    function find_program_address(bytes32 program, Seed[] memory seeds) external view returns (bytes32, uint8);
    function create_account(bytes32 owner, uint64 len, address user, bytes32 salt) external returns(bytes32);
    function allocate(bytes32 acc, uint64 space) external returns(bytes32);
    function assign(bytes32 acc, bytes32 owner) external returns(bytes32);
    function transfer(bytes32 to, uint64 amount, bytes32 salt) external returns(bytes32); 
    function program_id() external view returns(bytes32);
    function rome_evm_program_id() external view returns(bytes32);
    function bytes32_to_base58(bytes32) external view returns(bytes memory);
    function base58_to_bytes32(bytes memory) external view returns(bytes32);
}

address constant spl_token_address = address(0xff00000000000000000000000000000000000005);
address constant aspl_token_address = address(0xFF00000000000000000000000000000000000006);
address constant system_program_address = address(0xfF00000000000000000000000000000000000007);

ISplToken constant SplToken = ISplToken(spl_token_address);
IAssociatedSplToken constant AssociatedSplToken = IAssociatedSplToken(aspl_token_address);
ISystemProgram constant SystemProgram = ISystemProgram(system_program_address);


// File contracts/shared.sol

// Original license: SPDX_License_Identifier: MIT
pragma solidity ^0.8.20;

contract Shared {
    error SafeCastOverflowedUintDowncast(uint8 bits, uint256 value);


    function bytes32_to_bytes(bytes32 src) internal pure returns (bytes memory) {
        bytes memory dst = new bytes(32);

        assembly {
            mstore(add(dst, 32), src)
        }
        return dst;
    }

    function uint_to_bytes(uint src) internal pure returns (bytes memory) {
        bytes memory dst = new bytes(32);

        assembly {
            mstore(add(dst, 32), src)
        }
        return dst;
    }

    function bytes_to_bytes32(bytes memory src) internal pure returns (bytes32) {
        bytes32 dst;

        assembly {
            dst := mload(add(src, 32))
        }
        return  dst;
    }

    function chain_id_le(uint chain_id) internal pure returns (bytes memory ) {
        return abi.encodePacked(
                            uint8( chain_id),
                            uint8(chain_id >> 8 ),
                            uint8(chain_id >> 16),
                            uint8(chain_id >> 24),
                            uint8(chain_id >> 32),
                            uint8(chain_id >> 40),
                            uint8(chain_id >> 48),
                            uint8(chain_id >> 56)
                            );
    }

    function balance_key_seeds(address user, uint chain_id) internal pure returns(ISystemProgram.Seed[] memory) {
        bytes memory chain_le = chain_id_le(chain_id);

        ISystemProgram.Seed[] memory seeds = new ISystemProgram.Seed[](3);
        seeds[0] = ISystemProgram.Seed(chain_le);
        seeds[1] = ISystemProgram.Seed(bytes("ACCOUN_SEED"));
        seeds[2] = ISystemProgram.Seed(abi.encodePacked(user));

        return seeds;
    }
    
    function revert_msg(bytes memory _returnData) internal pure returns (string memory) {
        if (_returnData.length < 68) return '';

        bytes memory mes;
        assembly {
            mes := add(_returnData, 0x04)
        }
        return abi.decode(mes, (string));
    }

    function toUint64(uint256 value) internal pure returns (uint64) {
        if (value > type(uint64).max) {
            revert SafeCastOverflowedUintDowncast(64, value);
        }
        return uint64(value);
    }
}


// File contracts/spl_transfer.sol

// Original license: SPDX_License_Identifier: MIT
pragma solidity ^0.8.20;


contract SplHolderT is Ownable, Shared {
    // event Message(string account);
    constructor() Ownable(msg.sender) {
    }

    function transfer(bytes32 from, bytes32 to, uint64 amount) onlyOwner public {
        ISplToken.Seed[] memory seeds = new ISplToken.Seed[](0);
        (bool success, bytes memory result) = spl_token_address.call(
            abi.encodeWithSignature("transfer(bytes32,bytes32,uint64,(bytes)[])", from, to, amount, seeds)
        );

        require (success, string(Shared.revert_msg(result)));
    }
}
