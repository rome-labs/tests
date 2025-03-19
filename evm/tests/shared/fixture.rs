use super::utils::{run_on_testnet, run_on_devnet};
use super::{client::Client, client_config, genesis_wallet, CLIENT_CONFIG_FEE_FREE_PATH, CLIENT_CONFIG_PATH, DEPOSIT_VALUE, DEVNET_CONFIG_FEE_FREE_PATH, DEVNET_CONFIG_PATH, TESTNET_CONFIG_FEE_FREE_PATH, TESTNET_CONFIG_PATH};
use ethers::prelude::*;
use ethers_signers::{Signer as EthSigner, };
use rand::prelude::*;
use rstest::fixture;
use std::sync::Arc;
use ethers_core::types::transaction::eip2718::TypedTransaction;
use ethers_core::types::transaction::optimism::DepositTransaction;

#[fixture]
#[once]
pub fn client(#[default(true)] zero_gas: bool) -> Arc<Client> {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let path = cfg_path(zero_gas);
    let mut config = client_config(path);

    if !run_on_testnet() && !run_on_devnet() {
        let mut rng = rand_core::OsRng {};
        let chain: u32 = rng.gen();
        config.chain_id = chain.into();
    } else {
        println!("Chain ID fixture: {}", config.chain_id)
    }

    let user_wallet = genesis_wallet();

    if !run_on_testnet() && !run_on_devnet() {
        // deposit tx
        let rlp = rlp_0x7e(user_wallet.address());

        runtime.block_on(async {
            let client = Client::new(config, user_wallet).await;
            client
                .reg_owner(
                    client.chain_id(),
                    &client.upgrade_authority,
                )
                .await
                .unwrap();

            client
                .deposit(rlp.as_ref(), &client.user_solana_wallet)
                .await
                .unwrap();
            Arc::new(client)
        })
    } else {
        runtime.block_on(async {
            let client = Client::new(config, user_wallet).await;
            Arc::new(client)
        })
    }
}

pub fn rlp_0x7e(address: Address) -> Bytes {

    let tx = DepositTransaction {
        tx: TransactionRequest {
            from: Some(address),
            to: Some(address.into()),
            gas: Some(21000.into()),
            gas_price: None,
            value: Some(DEPOSIT_VALUE.into()),
            data: None,
            nonce: None,
            chain_id: None,
        },
        source_hash: H256::zero(),
        mint: Some(DEPOSIT_VALUE.into()),
        is_system_tx: false,
    };
    let sig = Signature {
        r: U256::default(),
        s: U256::default(),
        v: 0,
    };

    let typed_tx: TypedTransaction = tx.clone().into();
    let rlp = typed_tx.rlp_signed(&sig);

    rlp
}

pub fn cfg_path (zero_gas: bool) -> &'static str {
    if run_on_testnet() {
        if zero_gas {
            TESTNET_CONFIG_FEE_FREE_PATH
        } else {
            TESTNET_CONFIG_PATH
        }
    } else if run_on_devnet() {
        if zero_gas {
            DEVNET_CONFIG_FEE_FREE_PATH
        } else {
            DEVNET_CONFIG_PATH
        }
    } else {
        if zero_gas {
            CLIENT_CONFIG_FEE_FREE_PATH
        } else {
            CLIENT_CONFIG_PATH
        }
    }
}
