use {
    super::{client_config, wallet, CREATE_BALANCE_VALUE},
    ethers::prelude::*,
    ethers_core::k256::ecdsa::SigningKey,
    ethers_signers::{Signer as EthSigner, Wallet},
    rome_evm_client::{
        emulator::{emulate, Instruction::CreateBalance},
        RomeEVMClient as Client,
    },
    rstest::fixture,
    solana_client::rpc_client::RpcClient,
    solana_program::instruction::{AccountMeta, Instruction},
    solana_sdk::{
        commitment_config::{CommitmentConfig, CommitmentLevel},
        signature::{read_keypair_file, Signer},
    },
    std::{path::Path, sync::Arc},
    tokio_util::sync::CancellationToken,
};

#[fixture]
#[once]
pub fn client() -> Client {
    let config = client_config();
    let program_id = read_keypair_file(Path::new(&config.program_id_keypairs.get(0).unwrap()))
        .expect("read program_id keypair error")
        .pubkey();
    let payer = Arc::new(
        read_keypair_file(Path::new(&config.payer_keypair)).expect("read payer keypair error"),
    );
    let client = Arc::new(RpcClient::new_with_commitment(
        &config.solana_url,
        CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        },
    ));

    let token = CancellationToken::new();

    Client::new(
        config.chain_id,
        program_id,
        payer,
        client,
        config.number_holders,
        config.commitment_level,
        token,
    )
}

#[fixture]
#[once]
pub fn owner_wallet(client: &Client) -> Wallet<SigningKey> {
    let blockhash = client.client.get_latest_blockhash().unwrap();

    let config = client_config();
    let contract_owner = read_keypair_file(Path::new(&config.contract_owner_keypair))
        .expect("read contract_owner keypair error");

    let owner_wallet = wallet();
    let owner_addr = Address::from_slice(owner_wallet.address().as_bytes());

    let value: evm::U256 = CREATE_BALANCE_VALUE.into();
    let mut buf = [0; 32];
    value.to_big_endian(&mut buf);

    let mut data = vec![CreateBalance as u8];
    data.extend(owner_addr.as_bytes());
    data.extend(buf);

    let emulation = emulate(
        &client.program_id,
        &data,
        &contract_owner.pubkey(),
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

    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[ix],
        Some(&contract_owner.pubkey()),
        &[&contract_owner],
        blockhash,
    );

    let _ = client.client.send_and_confirm_transaction(&tx).unwrap();
    let balance = client.get_balance(owner_addr).unwrap();
    assert_eq!(value.as_u64(), balance.as_u64());

    owner_wallet
}

// #[fixture]
// #[once]
// pub fn owner() -> Wallet<SigningKey> {
//     let bin = hex::decode(OWNER_WALLET).unwrap();
//     Wallet::from_bytes(&bin).unwrap()
// }
