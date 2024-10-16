use ethereum_abi::{Abi, Value};
use ethers::types::TransactionRequest;
use ethers_core::k256::ecdsa::SigningKey;
use ethers_core::types::{
    transaction::eip2718::TypedTransaction, Address, Eip1559TransactionRequest, NameOrAddress, U256,
};
use ethers_signers::{Signer, Wallet};
use rome_sdk::rome_evm_client::RomeEVMClient as Client;
use solana_program::keccak::hash;

#[allow(dead_code)]
pub fn calc_address(client: &Client, from: &Address) -> Address {
    let from = Address::from_slice(from.as_bytes());
    let nonce = client.transaction_count(from).unwrap().as_u64();
    let nonce: U256 = nonce.into();

    let mut rlp = rlp::RlpStream::new_list(2);
    rlp.append(&from);
    rlp.append(&nonce);
    let hash = hash(&rlp.out());
    let address: [u8; 20] = hash.to_bytes()[12..].try_into().unwrap();
    Address::from(address)
}

#[allow(dead_code)]
pub fn do_tx(
    client: &Client,
    to: Option<Address>,
    data: Vec<u8>,
    wallet: &Wallet<SigningKey>,
    value: u64,
    tx_type: u8,
) -> TypedTransaction {
    let from = Address::from_slice(wallet.address().as_bytes());
    let nonce = client.transaction_count(from).unwrap().as_u64();
    println!("nonce: {}", nonce);

    match tx_type {
        0 => {
            let mut legacy = TransactionRequest {
                to: to.map(|a| NameOrAddress::Address(Address::from_slice(a.as_bytes()))),
                data: Some(data.into()),
                nonce: Some(nonce.into()),
                chain_id: Some(client.chain_id().into()),
                gas_price: Some(1.into()),
                value: Some(value.into()),
                ..Default::default()
            };
            legacy.from = Some(from);
            legacy.gas = Some(client.estimate_gas(&legacy).unwrap());
            TypedTransaction::Legacy(legacy)
        },
        2 => {
            let mut eip1559 = Eip1559TransactionRequest {
                to: to.map(|a| NameOrAddress::Address(Address::from_slice(a.as_bytes()))),
                data: Some(data.into()),
                nonce: Some(nonce.into()),
                chain_id: Some(client.chain_id().into()),
                value: Some(value.into()),
                max_priority_fee_per_gas: Some(1.into()), // TODO: do not use it
                max_fee_per_gas: Some(1.into()),
                ..Default::default()
            };
            let mut legacy: TransactionRequest = eip1559.clone().into();
            legacy.from = Some(from);
            eip1559.gas = Some(client.estimate_gas(&legacy).unwrap());
            TypedTransaction::Eip1559(eip1559)
        },
        _ => unimplemented!()
    }
}

// pub fn method_id(name: &str) -> [u8; 4] {
//     let hash = hash(name.as_bytes()).to_bytes();
//     hash[0..4].try_into().unwrap()
// }

#[allow(dead_code)]
pub fn abi(path: &str) -> Abi {
    let bin = std::fs::read(path).unwrap();
    let str = String::from_utf8(bin).unwrap();
    let abi: Abi = serde_json::from_str(&str).unwrap();
    abi
}

#[allow(dead_code)]
pub fn method_id(abi: &Abi, method: &str) -> Vec<u8> {
    let arg_split: Vec<&str> = method.split(&['(', ' ', ')']).collect();
    assert!(arg_split.len() == 1 || arg_split.len() == 4);
    let mut arg = if arg_split.len() == 4 {
        println!("{:?}", arg_split);
        match arg_split[1] {
            "uint256" => {
                let val: u64 = arg_split[2].parse().unwrap();
                let val_u256 = primitive_types::U256::from(val);
                Value::encode(&[Value::Uint(val_u256, 32)])
            },
            "string" => {
                let val = arg_split[2].to_string();
                Value::encode(&[Value::String(val)])
            },
            _ => unimplemented!(),
        }
    } else {
        vec![]
    };

    let method = abi
        .functions
        .iter()
        .filter(|a| a.name == arg_split[0].to_string())
        .next()
        .unwrap();

    let mut bin = method.method_id().to_vec();
    bin.append(&mut arg);
    bin
}
