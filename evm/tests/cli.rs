mod shared;

use {
    ethereum_abi::Value,
    ethers_core::types::Bytes,
    ethers_signers::Signer,
    rome_evm_client::{
        emulator::{Emulation, Instruction::DoTx},
        RomeEVMClient as Client,
    },
    rstest::*,
    shared::{
        client,
        tx::{abi, calc_address, do_rlp, method_id, wallet},
        CONTRACTS,
    },
};

pub fn emulate_do_tx(client: &Client, rlp: Vec<u8>) -> Emulation {
    let emulation = client.emulate(DoTx, &rlp).unwrap();
    println!("");
    for (key, acc) in &emulation.accounts {
        println!("\npubkey: {} {:?}", key, acc);
    }
    // println!("");
    // for (key, acc) in &map {
    //     println!("\npubkey: {}", key);
    //     println!("data: {:?}", hex::encode(&acc.account.data));
    // }
    emulation
}

#[rstest(
    contract,
    case::storage("Storage"),
    case::hello_world("HelloWorld")
)]
async fn deploy_contract_cli(client: &Client, contract: String) {
    let path = format!("{}{}.binary", CONTRACTS, contract);
    let bin = std::fs::read(&path).unwrap();
    let wallet = wallet();
    let rlp = do_rlp(client, None, bin, &wallet);
    let _accounts = client.emulate(DoTx, &rlp).unwrap().accounts;
    // assert_eq!(accounts.len(), 5);
}

#[rstest(
    contract,
    methods,
    case::storage("Storage", vec!["change", "get", "get_local", "add", "get_text", "update_text", "deploy"]),
    case::update_storage("UpdateStorage", vec!["update"]),
)]
async fn call_contract_cli(client: &Client, contract: String, methods: Vec<&str>) {
    let bin_path = format!("{}{}.binary", CONTRACTS, contract);
    let abi_path = format!("{}{}.abi", CONTRACTS, contract);

    // create wallet
    let wallet = wallet();
    //deploy contract
    let bin = std::fs::read(&bin_path).unwrap();
    let rlp = do_rlp(client, None, bin, &wallet);
    client
        .send_transaction(Bytes::from(rlp.clone()))
        .await
        .unwrap();

    let to = calc_address(client, &wallet.address());
    let abi = abi(&abi_path);

    for func in methods {
        let rlp = do_rlp(client, Some(to), method_id(&abi, &func), &wallet);
        let _accounts = emulate_do_tx(client, rlp).accounts;
        // assert_eq!(accounts.len(), 3);
    }
}

#[rstest(
    contract,
    caller,
    methods,
    case::storage("Storage", "StorageCaller", vec!["change", "get", "get_local", "add", "get_text", "update_text"]),
)]
async fn contract_caller_cli(client: &Client, contract: String, caller: String, methods: Vec<&str>) {
    let bin_path = format!("{CONTRACTS}{contract}.binary");
    let wallet = wallet();

    //deploy contract
    let mut bin = std::fs::read(&bin_path).unwrap();
    let to = primitive_types::H160::from_slice(wallet.address().as_bytes());
    let mut ctor = Value::encode(&[Value::Address(to)]);
    bin.append(&mut ctor);
    let rlp = do_rlp(client, None, bin, &wallet);
    client
        .send_transaction(Bytes::from(rlp.clone()))
        .await
        .unwrap();

    let abi_path = format!("{CONTRACTS}{caller}.abi");
    let to = calc_address(client, &wallet.address());
    let abi = abi(&abi_path);

    for func in methods {
        let rlp = do_rlp(&client, Some(to), method_id(&abi, &func), &wallet);
        let _accounts = emulate_do_tx(client, rlp).accounts;
        // assert_eq!(accounts.len(), 4);
    }
}
