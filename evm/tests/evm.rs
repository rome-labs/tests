mod shared;

use {
    ethereum_abi::Value,
    ethers_core::types::{Address, Bytes},
    ethers_signers::Signer as EthSigner,
    evm::U256,
    primitive_types::H160,
    rome_evm_client::{
        emulator::Instruction::CreateBalance,
        RomeEVMClient as Client,
    },
    rstest::*,
    shared::{
        client,
        tx::{abi, calc_address, do_rlp, method_id, wallet},
        CONTRACTS,
    },
    solana_program::instruction::{AccountMeta, Instruction},
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
    let rlp = do_rlp(&client, None, bin, &wallet);
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
    let rlp = do_rlp(&client, None, bin, &wallet);
    client
        .send_transaction(Bytes::from(rlp.clone()))
        .await
        .unwrap();

    let to = calc_address(&client, &wallet.address());
    let abi = abi(&abi_path);

    for func in methods {
        let rlp = do_rlp(&client, Some(to), method_id(&abi, &func), &wallet);
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
    let rlp = do_rlp(&client, None, bin, &wallet);
    client
        .send_transaction(Bytes::from(rlp.clone()))
        .await
        .unwrap();

    let abi_path = format!("{CONTRACTS}{caller}.abi");
    let to = calc_address(&client, &wallet.address());
    let abi = abi(&abi_path);

    for func in methods {
        let rlp = do_rlp(&client, Some(to), method_id(&abi, &func), &wallet);
        client
            .send_transaction(Bytes::from(rlp.clone()))
            .await
            .unwrap();
    }
}

#[ignore]
#[rstest]
fn create_balance(client: &Client) {
    let blockhash = client.client.get_latest_blockhash().unwrap();

    let payer = read_keypair_file(Path::new("/opt/ci/rome-owner-keypair.json"))
        .expect("read payer keypair error");

    let bin = hex::decode("5B38Da6a701c568545dCfcB03FcB875f56beddC4").unwrap();
    let address = Address::from_slice(&bin);
    let value: U256 = 123.into();
    let mut buf = [0; 32];
    value.to_big_endian(&mut buf);

    let mut data = vec![CreateBalance as u8];
    data.extend(address.as_bytes());
    data.extend(buf);

    let emulation = emulator::emulate(
        &client.program_id,
        &data,
        &payer.pubkey(),
        client.client.clone(),
    )
    .unwrap();

    let mut meta = vec![];
    for (key, item) in &emulation.accounts {
        if item.writable {
            meta.push(AccountMeta::new(*key, false))
        } else {
            meta.push(AccountMeta::new_readonly(*key, false))
        }
    }

    let ix = Instruction::new_with_bytes(client.program_id, &data, meta);

    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);

    let _ = client.client.send_and_confirm_transaction(&tx).unwrap();
    let balance = client.get_balance(address).unwrap();

    assert_eq!(value.as_u64(), balance.as_u64());
}
