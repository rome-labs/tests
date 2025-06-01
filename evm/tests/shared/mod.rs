use config::{load_config, Config};
use ethers::prelude::k256::SecretKey;
use ethers_core::k256::ecdsa::SigningKey;
use ethers_signers::Wallet;
use hex::decode;
use rome_sdk::rome_solana::payer::SolanaKeyPayer;
use utils::{run_on_devnet, run_on_testnet};
use solana_program::pubkey::Pubkey;
use solana_sdk::signer::Signer;

pub mod client;
pub mod config;
pub mod fixture;
pub mod tx;
pub mod utils;

pub const CONTRACTS: &'static str = "/opt/solidity/";
pub const RHEA_CONFIG_PATH: &'static str = "/opt/ci/cfg/rhea-config.yml";
pub const PROXY_CONFIG_PATH: &'static str = "/opt/ci/cfg/proxy-config.yml";
pub const CLIENT_CONFIG_PATH: &'static str = "/opt/ci/cfg/client-config.yaml";
pub const CLIENT_CONFIG_FEE_FREE_PATH: &'static str = "/opt/ci/cfg/client-config-fee-free.yaml";
pub const TESTNET_CONFIG_PATH: &'static str = "/opt/ci/cfg/testnet-config.yaml";
pub const TESTNET_CONFIG_FEE_FREE_PATH: &'static str = "/opt/ci/cfg/testnet-config-fee-free.yaml";
pub const DEVNET_CONFIG_PATH: &'static str = "/opt/ci/cfg/devnet-config.yaml";
pub const DEVNET_CONFIG_FEE_FREE_PATH: &'static str = "/opt/ci/cfg/devnet-config-fee-free.yaml";
pub const WITHDRAW_ACCOUNT_PATH: &'static str = "/opt/ci/keys/empty-keypair.json";
pub const DEPOSIT_VALUE: u128 = 1_000_000_000_000_000_000_000;
pub const RECEIVER_PK: &'static str =
    "0x4c0883a69102937d6231471b5dbb6204fe512961708279f1d7e5e8a4b5c5e3c4";
#[allow(dead_code)]
pub const ASSOCIATED_TOKEN_ACCOUNT_PROGRAM: Pubkey =
    Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
#[allow(dead_code)]
pub const SPL_TOKEN_ID: Pubkey =
    Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
#[allow(dead_code)]
pub const WITHDRAWAL_ADDRESS: &str = "0x4200000000000000000000000000000000000016"; 
#[allow(dead_code)]
pub const MINT_ADDRESS: &str = "D1TGfv7KTNRroYqcpesuDafJ7a85mEuLCTkEA8W8NBdW";     
pub fn client_config(path: &str) -> Config {
    load_config(path).unwrap()
}

#[allow(dead_code)]
pub fn test_account() -> Pubkey {
    let path = std::path::PathBuf::from(WITHDRAW_ACCOUNT_PATH);
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let keypair = runtime.block_on(async {
         SolanaKeyPayer::read_from_file(&path)
             .await
             .unwrap()
             .into_keypair()
    });
    
    keypair.pubkey()
}

#[allow(dead_code)]
pub fn wallet() -> ethers_signers::Wallet<ethers_core::k256::ecdsa::SigningKey> {
    if !run_on_devnet() && !run_on_testnet() {
        let mut rng = rand_core::OsRng {};
        ethers_signers::Wallet::new(&mut rng)
    } else {
        genesis_wallet()
    }
}

#[allow(dead_code)]
fn wallet_from_private_key(private_key: &str) -> Wallet<SigningKey> {
    let private_key_bytes = decode(private_key).expect("Invalid hex string");
    let secret_key = SecretKey::from_slice(&private_key_bytes).expect("Invalid private key");
    Wallet::from(secret_key)
}

#[allow(dead_code)]
pub fn genesis_wallet() -> Wallet<SigningKey> {
    let owner_wallet = wallet_from_private_key(genesis_private_key());
    owner_wallet
}

#[allow(dead_code)]
pub fn genesis_private_key() -> &'static str {
    "3f37802575d0840281551d5619256a84762e8236325537e8818730082645be65"
}
