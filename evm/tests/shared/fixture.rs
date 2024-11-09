use super::{client::Client, client_config, CREATE_BALANCE_VALUE, CLIENT_CONFIG_PATH,
            CLIENT_CONFIG_FEE_FREE_PATH
};
use ethers::prelude::*;
use ethers_core::types::Address;
use ethers_signers::Signer as EthSigner;
use rand::prelude::*;
use rstest::fixture;
use solana_sdk::signature::Signer;
use std::sync::Arc;

#[allow(dead_code)]
pub fn client_new_chain(zero_gas: bool) -> Arc<Client> {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let path = if zero_gas {
        CLIENT_CONFIG_FEE_FREE_PATH
    } else {
        CLIENT_CONFIG_PATH
    };

    let mut config = client_config(path);
    let mut rng = rand_core::OsRng {};
    let chain: u32 = rng.gen();
    config.chain_id = chain.into();

    let rollup_owner_wallet = super::genesis_wallet();
    let address = Address::from_slice(rollup_owner_wallet.address().as_bytes());

    runtime.block_on(async {
        let client = Client::new(config, rollup_owner_wallet).await;
        client
            .reg_owner(
                &client.rollup_owner.pubkey(),
                client.chain_id(),
                &client.upgrade_authority,
            )
            .await
            .unwrap();
        client
            .create_balance(address, CREATE_BALANCE_VALUE.into(), &client.rollup_owner)
            .await
            .unwrap();
        Arc::new(client)
    })
}

#[fixture]
#[once]
pub fn client() -> Client {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let config = client_config(CLIENT_CONFIG_PATH);
    let rollup_owner_wallet = super::genesis_wallet();
    let address = Address::from_slice(rollup_owner_wallet.address().as_bytes());

    runtime.block_on(async {
        let client = Client::new(config, rollup_owner_wallet).await;
        client
            .reg_owner(
                &client.rollup_owner.pubkey(),
                client.chain_id(),
                &client.upgrade_authority,
            )
            .await
            .unwrap();
        client
            .create_balance(address, CREATE_BALANCE_VALUE.into(), &client.rollup_owner)
            .await
            .unwrap();
        client
    })
}

