mod shared;

use ethereum_abi::Value;
use primitive_types::H160;
use rome_sdk::rome_evm_client::emulator::Instruction;
use rstest::*;
use shared::{
    fixture::{client_new_chain, client},
    tx::{abi, do_tx, do_tx_base, do_rlp, method_id},
    wallet, CONTRACTS,
};
use ethers_core::k256::ecdsa::SigningKey;
use ethers_signers::{Signer as EthSigner, Wallet};
use crate::shared::client::Client;
use solana_sdk::signer::Signer;


#[rstest(
    contract,
    tx_type,
    zero_gas,
    case::hello_world("HelloWorld", vec![0, 2], true),
    case::touch_storage("TouchStorage", vec![0, 2], true),
    case::touch_storage("TouchStorage", vec![0, 2], false),
    case::huge("uniswap/Huge", vec![2], true),
)]
async fn evm_deploy(contract: String, tx_type: Vec<u8>, zero_gas: bool) {
    let wallet = wallet();
    let client = client_new_chain(zero_gas);

    client.airdrop(wallet.address(), 1_000_000_000_000, zero_gas).await;
    for typ in tx_type {
        client.deploy(&contract, &wallet, None, typ, zero_gas).await;
    }
}

#[rstest(
    contract,
    methods,
    zero_gas,
    case::cu(
        "CU",
        vec![
            "update",
            "update_single",
            "push",
        ],
        false
    ),
    case::tstore(
        "TStore",
        vec![
            "check(uint256 1)",
            "check(uint256 2)",
            "check(uint256 5)",
        ],
        false
    ),
    case::selfdestruct(
        "DestructCaller",
        vec![
            "deploy",
            "deploy_and_destruct",
            "check",
        ],
        false
    ),
)]
async fn evm_call_unchecked(
    contract: String,
    methods: Vec<&str>,
    zero_gas: bool,
) {
    let client = client_new_chain(zero_gas);

    let wallet = wallet();
    client.airdrop(wallet.address(), 1_000_000_000_000, zero_gas).await;
    // deploy contract
    let address = client.deploy(&contract, &wallet, None, 2, zero_gas).await;
    // update storage
    for method in methods {
        client.method_call(&contract, &address, method, &wallet, 2, zero_gas).await
    }
}

#[rstest(
    contract,
    methods,
    eth_calls,
    results,
    zero_gas,
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
        ],
        false
    ),
)]
async fn evm_call(
    contract: String,
    methods: Vec<&str>,
    eth_calls: Vec<&str>,
    results: Vec<&str>,
    zero_gas: bool,
) {
    let client = client_new_chain(zero_gas);

    let wallet = wallet();
    client.airdrop(wallet.address(), 1_000_000_000_000, zero_gas).await;
    // deploy contract
    let address = client.deploy(&contract, &wallet, None, 2, zero_gas).await;
    // update storage
    for method in methods {
        client.method_call(&contract, &address, method, &wallet, 2, zero_gas).await
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
    zero_gas,
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
        ],
        false,
    ),
)]
async fn evm_nested_call(
    contract: String,
    caller: String,
    methods: Vec<&str>,
    eth_calls: Vec<&str>,
    results: Vec<&str>,
    zero_gas: bool,
) {
    let client = client_new_chain(zero_gas);

    let wallet = wallet();
    client.airdrop(wallet.address(), 1_000_000_000_000, zero_gas).await;
    // deploy contract
    let contract_addr = client.deploy(&contract, &wallet, None, 2, zero_gas).await;
    // deploy caller
    let ctor = Value::encode(&[Value::Address(H160::from_slice(contract_addr.as_bytes()))]);
    let caller_addr = client.deploy(&caller, &wallet, Some(ctor), 2, zero_gas).await;
    // call nested contract's methods
    for method in methods {
        client.method_call(&caller, &caller_addr, method, &wallet, 2, zero_gas).await
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
    case::storage("AtomicIterative", vec!["atomic_rw", "iterative_rw"], 0),
    case::storage("AtomicIterative", vec!["atomic_ro", "iterative_ro"], 2),
)]
#[serial_test::serial]
async fn evm_gas_transfer(
    client: &Client,
    contract: String,
    methods: Vec<&str>,
    tx_type: u8
) {
    let wallet = wallet();
    client.airdrop(wallet.address(), 1_000_000_000_000, false).await;

    // // deploy contract
    let address = client.deploy(&contract, &wallet, None, tx_type, false).await;
    let abi = abi(&format!("{}{}.abi", CONTRACTS, contract));

    // call methods and compare the estimate gas with gas_transfer
    for method in methods {
        let tx = do_tx(&client, Some(address), method_id(&abi, method), &wallet, 0, tx_type);

        let before = client.get_balance(wallet.address()).unwrap();
        client.method_call(&contract, &address, method, &wallet, tx_type, false).await;
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
    zero_gas,
    case::get_storage_at("GetStorageAt", "get", 2, true),
)]
async fn evm_get_storage_at(
    contract: String,
    method: String,
    tx_type: u8,
    zero_gas: bool,
) {
    let client = client_new_chain(zero_gas);
    let wallet = wallet();
    client.airdrop(wallet.address(), 1_000_000_000_000, zero_gas).await;

    // deploy contract
    let address = client.deploy(&contract, &wallet, None, tx_type, zero_gas).await;

    // emulate the call of the contract method
    let abi = abi(&format!("{}{}.abi", CONTRACTS, contract));
    let tx = do_tx(&client, Some(address), method_id(&abi, &method), &wallet, 0, tx_type);
    let mut tx_data = vec![0]; // Option<fee_recipient>
    tx_data.append(&mut do_rlp(&tx, &wallet));

    let resource = client.tx_builder().lock_resource().await.unwrap();

    let emulation = client.emulate(Instruction::DoTx, &tx_data, &resource.payer().pubkey()).unwrap();

    // parse the emulation report
    assert_eq!(emulation.storage.len(), 1);
    let (slot_addr, slots) = emulation.storage.first_key_value().unwrap();
    assert_eq!(slot_addr.as_bytes(), address.as_bytes());
    assert_eq!(slots.len(), 1);
    let (slot, rw) = slots.first_key_value().unwrap();
    assert_eq!(slot.as_u64(), 0);
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

///  case1: Iterative tx writes to storage account. Atomic tx writes to the locked account. Error
///  case2: Iterative tx writes to storage account. Atomic tx reads the locked account. Ok
///  case3: Iterative tx writes to storage account. Iterative tx writes to the locked account. Error
///  case4: Iterative tx writes to storage account. Iterative tx reads the locked account. Error

///  case5: Iterative tx reads storage account. Atomic tx reads the locked account. Ok
///  case6: Iterative tx reads storage account. Atomic tx writes to the locked account. Error
///  case7: Iterative tx reads storage account. Iterative tx writes to the locked account. Error
///  case8: Iterative tx reads storage account. Iterative tx reads the locked account. Result: Ok
#[rstest(
    contract,
    first,
    first_count,
    second,
    second_count,
    tx_type,
#[should_panic]
    case::iter_rw_atomic_rw("AtomicIterative", "iterative_rw", 5, "atomic_rw", 20, 2),
    case::iter_rw_atomic_ro("AtomicIterative", "iterative_rw", 5, "atomic_ro", 20, 2),
#[should_panic]
    case::iter_rw_iter_rw("AtomicIterative", "iterative_rw", 5, "iterative_rw", 5, 2),
#[should_panic]
    case::iter_rw_iter_ro("AtomicIterative", "iterative_rw", 5, "iterative_ro", 5, 2),
    case::iter_ro_atomic_ro("AtomicIterative", "iterative_ro", 5, "atomic_ro",  20, 2),
#[should_panic]
    case::iter_ro_atomic_rw("AtomicIterative", "iterative_ro", 5, "atomic_rw",  20, 2),
#[should_panic]
    case::iter_ro_iter_rw("AtomicIterative", "iterative_ro", 5, "iterative_rw", 5, 2),
    case::iter_ro_iter_ro("AtomicIterative", "iterative_ro", 5, "iterative_ro", 5, 2),
)]
#[serial_test::serial]
async fn evm_account_lock(
    contract: String,
    first: &str,
    first_count: u64,
    second: &str,
    second_count: u64,
    tx_type: u8
) {
    let wallet1 = wallet();
    let wallet2 = wallet();
    let zero_gas = true;

    let client_zero_gas = client_new_chain(zero_gas);
    // deploy contract
    let address = client_zero_gas.deploy(&contract, &wallet1, None, tx_type, zero_gas).await;

    // preparing the method calls
    let abi = abi(&format!("{}{}.abi", CONTRACTS, contract));

    let f = |method: &str, count: u64, wallet: &Wallet<SigningKey>, | -> Vec<Vec<u8>> {
        let nonce = client_zero_gas.transaction_count(wallet.address()).unwrap().as_u64();
        let mut rlp = vec![];
        for i in 0..count {
            let tx = do_tx_base(&client_zero_gas, Some(address), method_id(&abi, method), &wallet, 0, tx_type, nonce + i);
            rlp.push(do_rlp(&tx, &wallet).to_vec()) ;
        }
        rlp
    };

    let rlp_first = f(first, first_count, &wallet1);
    let rlp_second = f(second, second_count, &wallet2);

    let client1 = client_zero_gas.clone();
    let client2 = client_zero_gas.clone();


    let tx1_jh = tokio::spawn(
        async move {
            for rlp in rlp_first {
                client1.send_transaction(rlp.into()).await.unwrap();
            }
        });

    let tx2_jh = tokio::spawn(
        async move {
            for rlp in rlp_second {
                client2.send_transaction(rlp.into()).await.unwrap();
            }
        });

    let (tx1_res, tx2_res) = tokio::join!(tx1_jh, tx2_jh);

    tx1_res.map_err(|e| println!("{:?}", e)).unwrap();
    tx2_res.map_err(|e| println!("{:?}", e)).unwrap();
}
