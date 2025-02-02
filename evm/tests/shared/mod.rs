use config::{load_config, Config};
use ethers_core::k256::ecdsa::SigningKey;
use ethers_signers::Wallet;
use ethers::prelude::k256::SecretKey;
use hex::decode;

pub mod client;
pub mod config;
pub mod fixture;
pub mod tx;

pub const CONTRACTS: &'static str = "/opt/solidity/";
pub const CLIENT_CONFIG_PATH: &'static str = "/opt/ci/cfg/client-config.yaml";
pub const CLIENT_CONFIG_FEE_FREE_PATH: &'static str = "/opt/ci/cfg/client-config-fee-free.yaml";
pub const CREATE_BALANCE_VALUE: u128 = 1_000_000_000_000_000_000_000;
pub fn client_config(path: &str) -> Config {
    load_config(path).unwrap()
}

#[allow(dead_code)]
pub fn wallet() -> ethers_signers::Wallet<ethers_core::k256::ecdsa::SigningKey> {
    let mut rng = rand_core::OsRng {};
    ethers_signers::Wallet::new(&mut rng)
}

#[allow(dead_code)]
fn wallet_from_private_key(private_key: &str) -> Wallet<SigningKey> {
    let private_key_bytes = decode(private_key).expect("Invalid hex string");
    let secret_key = SecretKey::from_slice(&private_key_bytes).expect("Invalid private key");
    Wallet::from(secret_key)
}

#[allow(dead_code)]
pub fn genesis_wallet() -> Wallet<SigningKey> {
    let owner_wallet = wallet_from_private_key(
        genesis_private_key()
    );
    owner_wallet
}

#[allow(dead_code)]
pub fn genesis_private_key() -> &'static str {
    "3f37802575d0840281551d5619256a84762e8236325537e8818730082645be65"
}

