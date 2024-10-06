pub mod config;
pub mod cross_rollup_fn;
pub mod fixture;
pub mod tx;

use ethers::prelude::k256::ecdsa::SigningKey;
use {
    config::{load_config, Config},
    ethers::prelude::*,
};

pub fn client_config() -> Config {
    let config: Config = load_config(CLIENT_CONFIG_PATH)
        .expect(&format!("load config error {}", CLIENT_CONFIG_PATH));

    config
}

pub fn wallet() -> Wallet<SigningKey> {
    let mut rng = rand_core::OsRng {};
    Wallet::new(&mut rng)
}

#[allow(dead_code)]
pub const CONTRACTS: &'static str = "/opt/solidity/";
pub const ROME_CONFIG_PATH: &str = "/opt/ci/rome-config.json";
pub const CLIENT_CONFIG_PATH: &'static str = "/opt/ci/client-config.yaml";
pub const CREATE_BALANCE_VALUE: u64 = 123;
