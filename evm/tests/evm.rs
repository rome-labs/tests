mod shared;

use ethereum_abi::Value;
use ethers_signers::{Signer as EthSigner,};
use primitive_types::H160;
use rome_sdk::rome_evm_client::emulator::Instruction;
use rstest::*;
use shared::{
    fixture::client, client::Client,
    tx::{abi, do_tx, method_id},
    wallet, CONTRACTS,
};

#[rstest(
    contract,
    tx_type,
    case::hello_world("HelloWorld", vec![0, 2]),
    case::touch_storage("TouchStorage", vec![0, 2]),
)]
#[serial_test::serial]
async fn evm_deploy(client: &Client, contract: String, tx_type: Vec<u8>) {
    let wallet = wallet();

    client.airdrop(wallet.address(), 1_000_000_000_000).await;
    for typ in tx_type {
        client.deploy(&contract, &wallet, None, typ).await;
    }
}

#[rstest(
    contract,
    methods,
    eth_calls,
    results,
    case::touch_storage(
        "TouchStorage",
        vec![
            "set_value(uint256 10)",
            "push_vec(uint256 3)",
            "push_vec(uint256 4)",
            "set_text(string hello)",
            "deploy",
        ],
        vec![
            "get_value",
            "get_vec(uint256 0)",
            "get_vec(uint256 1)",
            "get_text",
            "get_local",
        ],
        vec![
            "000000000000000000000000000000000000000000000000000000000000000a",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
            "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000568656c6c6f000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000005",
        ]
    ),
)]
#[serial_test::serial]
async fn evm_call(
    client: &Client,
    contract: String,
    methods: Vec<&str>,
    eth_calls: Vec<&str>,
    results: Vec<&str>,
) {
    let wallet = wallet();
    client.airdrop(wallet.address(), 1_000_000_000_000).await;
    // deploy contract
    let address = client.deploy(&contract, &wallet, None, 2).await;
    // update storage
    for method in methods {
        client.method_call(&contract, &address, method, &wallet, 2).await
    }
    // call eth_calls to check the results
    for (eth_call, expected_hex) in eth_calls.iter().zip(results) {
        let result = client.eth_call(&contract, &address, eth_call, &wallet);
        let expected = hex::decode(expected_hex).unwrap();
        assert_eq!(result.to_vec(), expected);
    }
}

#[rstest(
    contract,
    caller,
    methods,
    eth_calls,
    results,
    case::touch_storage(
        "TouchStorage",
        "NestedCall",
        vec![
            "set_value(uint256 10)",
            "push_vec(uint256 3)",
            "push_vec(uint256 4)",
            "set_text(string hello)",
            "deploy",
        ],
        vec![
            "get_value",
            "get_vec(uint256 0)",
            "get_vec(uint256 1)",
            "get_text",
            "get_local",
            "text",
        ],
        vec![
            "000000000000000000000000000000000000000000000000000000000000000a",
            "0000000000000000000000000000000000000000000000000000000000000003",
            "0000000000000000000000000000000000000000000000000000000000000004",
            "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000568656c6c6f000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000005",
            "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000c48656c6c6f5f776f726c64210000000000000000000000000000000000000000"
        ]
    ),
)]
#[serial_test::serial]
async fn evm_nested_call(
    client: &Client,
    contract: String,
    caller: String,
    methods: Vec<&str>,
    eth_calls: Vec<&str>,
    results: Vec<&str>,
) {
    let wallet = wallet();
    client.airdrop(wallet.address(), 1_000_000_000_000).await;
    // deploy contract
    let contract_addr = client.deploy(&contract, &wallet, None, 2).await;
    // deploy caller
    let ctor = Value::encode(&[Value::Address(H160::from_slice(contract_addr.as_bytes()))]);
    let caller_addr = client.deploy(&caller, &wallet, Some(ctor), 2).await;
    // call nested contract's methods
    for method in methods {
        client.method_call(&caller, &caller_addr, method, &wallet, 2).await
    }
    // call eth_calls to check the results
    for (eth_call, expected_hex) in eth_calls.iter().zip(results) {
        let result = client.eth_call(&caller, &caller_addr, eth_call, &wallet);
        let expected = hex::decode(expected_hex).unwrap();
        assert_eq!(result.to_vec(), expected);
    }
}

#[rstest(
    contract,
    methods,
    tx_type,
    case::storage("AtomicIterative", vec!["atomic", "iterative"], 0),
    case::storage("AtomicIterative", vec!["atomic", "iterative"], 2),
)]
#[serial_test::serial]
async fn evm_gas_transfer(
    client: &Client,
    contract: String,
    methods: Vec<&str>,
    tx_type: u8
) {
    let wallet = wallet();
    client.airdrop(wallet.address(), 1_000_000_000_000).await;

    // deploy contract
    let address = client.deploy(&contract, &wallet, None, tx_type).await;
    let abi = abi(&format!("{}{}.abi", CONTRACTS, contract));

    // call methods and compare the estimate gas with gas_transfer
    for method in methods {
        let tx = do_tx(client, Some(address), method_id(&abi, method), &wallet, 0, tx_type);

        let before = client.get_balance(wallet.address()).unwrap();
        client.method_call(&contract, &address, method, &wallet, tx_type).await;
        let after = client.get_balance(wallet.address()).unwrap();

        let estimage_gas = tx.gas().unwrap();
        let gas_transfer = before.checked_sub(after).unwrap();

        assert!(gas_transfer >= 0.into());
        assert!(gas_transfer <= *estimage_gas);
    }

}

#[rstest(
    contract,
    method,
    tx_type,
    case::get_storage_at("GetStorageAt", "get", 0),
    case::get_storage_at("GetStorageAt", "get", 2),
)]
#[serial_test::serial]
async fn evm_get_storage_at(client: &Client, contract: String, method: String, tx_type: u8) {
    let wallet = wallet();
    client.airdrop(wallet.address(), 1_000_000_000_000).await;

    // deploy contract
    let address = client.deploy(&contract, &wallet, None, tx_type).await;

    // emulate the call of the contract method
    let abi = abi(&format!("{}{}.abi", CONTRACTS, contract));
    let tx = do_tx(client, Some(address), method_id(&abi, &method), &wallet, 0, tx_type);
    let sig = wallet.sign_transaction_sync(&tx).unwrap();
    let rlp = tx.rlp_signed(&sig);
    let emulation = client.emulate(Instruction::DoTx, &rlp, &client.get_payer_pubkey()).unwrap();

    // parse the emulation report
    assert_eq!(emulation.storage.len(), 1);
    let (slot_addr, slots) = emulation.storage.first_key_value().unwrap();
    assert_eq!(*slot_addr, evm::H160::from_slice(address.as_bytes()));
    assert_eq!(slots.len(), 1);
    let (slot, rw) = slots.first_key_value().unwrap();
    assert_eq!(*slot, evm::U256::zero());
    assert_eq!(*rw, false);

    let mut buf = [0u8; 32];
    slot.to_big_endian(&mut buf);
    let slot = ethers_core::types::U256::from_big_endian(&buf);
    let addr = ethers_core::types::H160::from_slice(slot_addr.as_bytes());

    // load the storage slot from chain
    let slot_value = client.eth_get_storage_at(addr, slot).unwrap();

    // check storage slot value
    assert_eq!(slot_value, 7.into());
}
