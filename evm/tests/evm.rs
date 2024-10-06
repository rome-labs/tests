mod shared;

use rome_evm_client::emulator;
use {
    ethereum_abi::Value,
    ethers::prelude::k256::ecdsa::SigningKey,
    ethers_core::types::{Address, Bytes},
    ethers_signers::Signer as EthSigner,
    ethers_signers::Wallet,
    primitive_types::H160,
    rome_evm_client::{
        emulator::{Instruction, Instruction::RegSigner},
        RomeEVMClient as Client,
    },
    rstest::*,
    shared::{
        client_config,
        fixture::{client, owner_wallet},
        tx::{abi, calc_address, do_rlp, do_rlp_with_gas, method_id},
        wallet, CONTRACTS, CREATE_BALANCE_VALUE,
    },
    solana_sdk::{
        signature::{read_keypair_file, Signer},
        transaction::Transaction,
    },
    std::path::Path,
};

#[rstest(
    contract,
    case::hello_world("HelloWorld"),
    case::storage("Storage"),
    case::storage_standard("StorageStandard")
)]
async fn deploy_contract(client: &Client, contract: String) {
    let path = format!("{}{}.binary", CONTRACTS, contract);
    let bin = std::fs::read(&path).unwrap();
    let wallet = wallet();
    let rlp = do_rlp(&client, None, bin, &wallet, 0);
    client
        .send_transaction(Bytes::from(rlp.clone()))
        .await
        .unwrap();
}

#[rstest(
    contract,
    methods,
    case::storage("Storage", vec!["change", "get", "get_local", "add", "get_text", "update_text", "deploy"]),
    case::storage_standard("StorageStandard", vec!["store(uint256 1)", "retrieve"]),
    case::storage_standard("StorageStandard", vec!["store(uint256 2)", "retrieve"]),
    case::update_storage("UpdateStorage", vec!["update"]),
)]
async fn call_contract(client: &Client, contract: String, methods: Vec<&str>) {
    let bin_path = format!("{}{}.binary", CONTRACTS, contract);
    let abi_path = format!("{}{}.abi", CONTRACTS, contract);

    // calc "from" address
    let wallet = wallet();
    //deploy contract
    let bin = std::fs::read(&bin_path).unwrap();
    let rlp = do_rlp(&client, None, bin, &wallet, 0);
    client
        .send_transaction(Bytes::from(rlp.clone()))
        .await
        .unwrap();

    let to = calc_address(&client, &wallet.address());
    let abi = abi(&abi_path);

    for func in methods {
        let rlp = do_rlp(&client, Some(to), method_id(&abi, &func), &wallet, 0);
        client
            .send_transaction(Bytes::from(rlp.clone()))
            .await
            .unwrap();
    }
}

#[rstest(
    contract,
    caller,
    methods,
    case::storage("Storage", "StorageCaller", vec!["change", "get", "get_local", "add", "get_text", "update_text"]),
)]
async fn contract_caller(client: &Client, contract: String, caller: String, methods: Vec<&str>) {
    let bin_path = format!("{CONTRACTS}{contract}.binary");
    // calc "from" address
    let wallet = wallet();
    //deploy contract
    let mut bin = std::fs::read(&bin_path).unwrap();
    let to = H160::from_slice(wallet.address().as_bytes());
    let mut ctor = Value::encode(&[Value::Address(to)]);
    bin.append(&mut ctor);
    let rlp = do_rlp(&client, None, bin, &wallet, 0);
    client
        .send_transaction(Bytes::from(rlp.clone()))
        .await
        .unwrap();

    let abi_path = format!("{CONTRACTS}{caller}.abi");
    let to = calc_address(&client, &wallet.address());
    let abi = abi(&abi_path);

    for func in methods {
        let rlp = do_rlp(&client, Some(to), method_id(&abi, &func), &wallet, 0);
        client
            .send_transaction(Bytes::from(rlp.clone()))
            .await
            .unwrap();
    }
}

async fn airdrop(
    client: &Client,
    owner: &Wallet<SigningKey>,
    user: &Wallet<SigningKey>,
    value: u64,
) {
    let to = Address::from_slice(user.address().as_bytes());
    let rlp = do_rlp(&client, Some(to), vec![], &owner, value);
    client
        .send_transaction(Bytes::from(rlp.clone()))
        .await
        .unwrap();
}

#[rstest(contract, gas_estimate, case::storage("Storage", 1))]
async fn gas_transfer(
    client: &Client,
    owner_wallet: &Wallet<SigningKey>,
    contract: String,
    gas_estimate: u64,
) {
    let user = wallet();
    let user_addr = Address::from_slice(user.address().as_bytes());
    let owner_addr = Address::from_slice(owner_wallet.address().as_bytes());
    let random = wallet();
    let gas_recipient = Address::from_slice(&random.address().as_bytes());

    assert!(gas_estimate <= CREATE_BALANCE_VALUE);
    assert_eq!(
        CREATE_BALANCE_VALUE,
        client.get_balance(owner_addr).unwrap().as_u64()
    );

    airdrop(client, owner_wallet, &user, gas_estimate).await;

    assert_eq!(
        gas_estimate,
        client.get_balance(user_addr).unwrap().as_u64()
    );
    assert_eq!(
        CREATE_BALANCE_VALUE - gas_estimate,
        client.get_balance(owner_addr).unwrap().as_u64()
    );

    client.reg_gas_recipient(gas_recipient.clone()).unwrap();

    assert_eq!(0, client.get_balance(gas_recipient).unwrap().as_u64());

    let path = format!("{}{}.binary", CONTRACTS, contract);
    let bin = std::fs::read(&path).unwrap();
    let rlp = do_rlp_with_gas(&client, None, bin, &user, gas_estimate);
    client
        .send_transaction(Bytes::from(rlp.clone()))
        .await
        .unwrap();

    assert_eq!(
        gas_estimate,
        client.get_balance(gas_recipient).unwrap().as_u64()
    );
    assert_eq!(0, client.get_balance(user_addr).unwrap().as_u64());
}

#[rstest(contract, case::hello_world("HelloWorld"))]
async fn get_storage_at(client: &Client, contract: String) {
    let path = format!("{}{}.binary", CONTRACTS, contract);
    let bin = std::fs::read(&path).unwrap();
    let wallet = wallet();
    let rlp = do_rlp(&client, None, bin, &wallet, 0);

    let contract_addr = calc_address(client, &wallet.address());
    let emulation = client.emulate(Instruction::DoTx, &rlp).unwrap();
    client
        .send_transaction(Bytes::from(rlp.clone()))
        .await
        .unwrap();

    assert_eq!(emulation.storage.len(), 1);
    let (slot_addr, slots) = emulation.storage.first_key_value().unwrap();
    assert_eq!(*slot_addr, evm::H160::from_slice(contract_addr.as_bytes()));
    assert_eq!(slots.len(), 1);
    let (slot, rw) = slots.first_key_value().unwrap();
    assert_eq!(*slot, evm::U256::zero());
    assert_eq!(*rw, true);

    let mut buf = [0u8; 32];
    slot.to_big_endian(&mut buf);
    let slot = ethers_core::types::U256::from_big_endian(&buf);
    let addr = ethers_core::types::H160::from_slice(slot_addr.as_bytes());


    let slot_value = client.eth_get_storage_at(addr, slot).unwrap();

    assert_eq!(slot_value, 7.into());
}
