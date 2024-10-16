use super::{client::Client, client_config, CREATE_BALANCE_VALUE};
use ethers_signers::Signer as EthSigner;
use rstest::fixture;
use solana_sdk::signature::Signer;
use std::str::FromStr;
use ethers_core::types::Address;

#[fixture]
#[once]
pub fn client() -> Client {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let config = client_config();
    let rollup_owner_wallet = super::genesis_wallet();
    let address = Address::from_slice(rollup_owner_wallet.address().as_bytes());
    let gas_recipient = Address::from_str("0x229E93198d584C397DFc40024d1A3dA10B73aB32").unwrap();

    runtime.block_on(async {
        let client = Client::new(config, rollup_owner_wallet, gas_recipient).await;
        client
            .reg_owner(&client.rollup_owner.pubkey(), client.chain_id(), &client.upgrade_authority)
            .await
            .unwrap();
        client
            .create_balance(address, CREATE_BALANCE_VALUE.into(), &client.rollup_owner)
            .await
            .unwrap();
        client
            .reg_gas_recipient(gas_recipient, client.get_payer())
            .await
            .unwrap();
        client
    })
}

