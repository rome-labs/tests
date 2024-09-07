pub mod config;
pub mod tx;

use rstest::fixture;
use {
    anyhow::Context,
    config::{load_config, Config},
    ethers::{
        core::abi::Abi,
        prelude::*,
        signers::{LocalWallet, Signer as EtherSigner},
        types::{Address, U256},
    },
    rome_sdk::{EthSignedTxTuple, RemusTx, Rome, RomeConfig},
    rome_evm_client::RomeEVMClient as Client,
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::{CommitmentConfig, CommitmentLevel},
        signature::{read_keypair_file, Signer},
    },
    std::{path::Path, sync::Arc},
    tokio_util::sync::CancellationToken,
    transaction::eip2718::TypedTransaction,
};

#[fixture]
#[once]
pub fn client() -> Client {
    let config: Config = load_config(CLIENT_CONFIG_PATH)
        .expect(&format! ("load config error {}", CLIENT_CONFIG_PATH));

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

pub fn client_(chain_id: u64) -> Client {
    let config: Config =
        load_config(CLIENT_CONFIG_PATH).expect(&format!("load config error {}", CLIENT_CONFIG_PATH));

    let keypair_path = match chain_id {
        1001 => config
            .program_id_keypairs
            .get(0)
            .expect("Keypair for chain_id 1001 not found"),
        1002 => config
            .program_id_keypairs
            .get(1)
            .expect("Keypair for chain_id 1002 not found"),
        _ => panic!("Unsupported chain_id"),
    };

    let program_id = read_keypair_file(Path::new(&keypair_path))
        .expect("read program_id keypair error")
        .pubkey();
    let payer = Arc::new(
        read_keypair_file(Path::new(&config.payer_keypair)).expect("read payer keypair error"),
    );
    let client = solana_rpc_client(&config.solana_url);
    let token = CancellationToken::new();

    Client::new(
        chain_id,
        program_id,
        payer,
        client,
        config.number_holders,
        config.commitment_level,
        token,
    )
}

#[allow(dead_code)]
fn solana_rpc_client(solana_rpc_endpoint: &str) -> Arc<RpcClient> {
    Arc::new(RpcClient::new_with_commitment(
        &solana_rpc_endpoint,
        CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        },
    ))
}

#[allow(dead_code)]
pub async fn transact_token_on_rollup(
    sender_private_key: &str,
    receiver_public_key: &str,
    proxy_endpoint: &str,
    contract_address: &str,
    chain_id: u64,
) -> Result<(EthSignedTxTuple, (U256, U256)), Box<dyn std::error::Error>> {
    let rome_evm_client = client_(chain_id);
    let (signer_middleware, wallet) =
        create_signer_middelware(sender_private_key, proxy_endpoint, chain_id)?;

    let token_contract_address: Address = contract_address.parse()?;
    let abi: Abi = serde_json::from_str(abi::ERC20_ABI)?;
    let contract = Contract::new(token_contract_address, abi, signer_middleware.clone());
    let sender_address: Address = wallet.address();
    let recipient_address: Address = receiver_public_key.parse()?;

    let amount_to_sell = U256::from_dec_str("1000000000000000000")?;
    let binding = contract.method::<_, H256>("approve", (sender_address, amount_to_sell))?;
    let _ = binding.send().await?;
    let binding = contract.method::<_, H256>("transfer", (recipient_address, amount_to_sell))?;
    let mut tx = binding.tx;

    let nonce = rome_evm_client
        .transaction_count(sender_address)
        .expect("Failed to compute nonce.");

    match &mut tx {
        TypedTransaction::Legacy(_tx_request) => {
            panic!("Transaction type not supported.");
        }
        TypedTransaction::Eip2930(_tx_request) => {
            panic!("Transaction type not supported.");
        }
        TypedTransaction::Eip1559(tx_request) => {
            tx_request.chain_id = Some(U64::from(chain_id));
            tx_request.nonce = Some(U256::from(nonce.as_u64() as i64));
            tx_request.from = Some(sender_address);
        }
    };

    let sig: Signature = wallet
        .sign_transaction(&tx)
        .await
        .context("failed to sign transaction")?;

    let sender_balance = get_balance_on_contract(
        sender_private_key,
        contract_address,
        proxy_endpoint,
        &format!("{:?}", sender_address),
        chain_id,
    )
    .await?;
    let receiver_balance = get_balance_on_contract(
        sender_private_key,
        contract_address,
        proxy_endpoint,
        &format!("{:?}", recipient_address),
        chain_id,
    )
    .await?;

    Ok((
        EthSignedTxTuple::new(tx, sig),
        (sender_balance, receiver_balance),
    ))
}

#[allow(dead_code)]
pub async fn compose_and_send_tx(
    tx_tuple: Vec<EthSignedTxTuple>,
) -> Result<(), Box<dyn std::error::Error>> {
    let rome_config = RomeConfig::load_json(ROME_CONFIG_PATH.parse()?).await?;
    let rome = Rome::new_with_config(rome_config).await?;
    let remus_tx = RemusTx::new(tx_tuple);
    let rome_tx = rome.compose_cross_rollup_tx(remus_tx).await?;
    let _ = rome.send_and_confirm_tx(rome_tx).await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn get_balance_on_contract(
    wallet_private_key: &str,
    contract_address: &str,
    proxy_endpoint: &str,
    account_public_key: &str,
    chain_id: u64,
) -> Result<U256, Box<dyn std::error::Error>> {
    let (client, _) = create_signer_middelware(wallet_private_key, proxy_endpoint, chain_id)?;

    let account_public_key: Address = account_public_key.parse()?;
    let token_contract_address: Address = contract_address.parse()?;

    let abi: Abi = serde_json::from_str(abi::ERC20_ABI)?;
    let contract = Contract::new(token_contract_address, abi, client.clone());

    let balance: U256 = contract
        .method::<_, U256>("balanceOf", account_public_key)?
        .call()
        .await?;

    Ok(balance)
}

#[allow(dead_code)]
pub fn create_signer_middelware(
    private_key: &str,
    proxy_endpoint: &str,
    chain_id: u64,
) -> Result<
    (
        Arc<SignerMiddleware<SignerMiddleware<Provider<Http>, LocalWallet>, LocalWallet>>,
        LocalWallet,
    ),
    Box<dyn std::error::Error>,
> {
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?;
    let provider = Provider::<Http>::try_from(proxy_endpoint)?;
    let provider = SignerMiddleware::new(provider.clone(), wallet.clone());
    let signer_middleware = SignerMiddleware::new(provider, wallet.clone().with_chain_id(chain_id));

    return Ok((Arc::new(signer_middleware), wallet));
}
pub mod abi {
    pub const ERC20_ABI: &str = r#"[{"inputs":[],"payable":false,"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"owner","type":"address"},{"indexed":true,"internalType":"address","name":"spender","type":"address"},{"indexed":false,"internalType":"uint256","name":"value","type":"uint256"}],"name":"Approval","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"from","type":"address"},{"indexed":true,"internalType":"address","name":"to","type":"address"},{"indexed":false,"internalType":"uint256","name":"value","type":"uint256"}],"name":"Transfer","type":"event"},{"constant":true,"inputs":[],"name":"DOMAIN_SEPARATOR","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"PERMIT_TYPEHASH","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"internalType":"address","name":"","type":"address"},{"internalType":"address","name":"","type":"address"}],"name":"allowance","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"spender","type":"address"},{"internalType":"uint256","name":"value","type":"uint256"}],"name":"approve","outputs":[{"internalType":"bool","name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"balanceOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"decimals","outputs":[{"internalType":"uint8","name":"","type":"uint8"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"name","outputs":[{"internalType":"string","name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"nonces","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"owner","type":"address"},{"internalType":"address","name":"spender","type":"address"},{"internalType":"uint256","name":"value","type":"uint256"},{"internalType":"uint256","name":"deadline","type":"uint256"},{"internalType":"uint8","name":"v","type":"uint8"},{"internalType":"bytes32","name":"r","type":"bytes32"},{"internalType":"bytes32","name":"s","type":"bytes32"}],"name":"permit","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"symbol","outputs":[{"internalType":"string","name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"totalSupply","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"value","type":"uint256"}],"name":"transfer","outputs":[{"internalType":"bool","name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"from","type":"address"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"value","type":"uint256"}],"name":"transferFrom","outputs":[{"internalType":"bool","name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"}]"#;
}

#[allow(dead_code)]
pub const CONTRACTS: &'static str = "/opt/solidity/";
pub const ROME_CONFIG_PATH: &str = "/opt/ci/rome-config.json";
pub const CLIENT_CONFIG_PATH: &'static str = "/opt/ci/client-config.yaml";
#[allow(dead_code)]
pub const CROSS_ROLLUP_CONFIG_PATH: &str = "/opt/ci/cross-rollup-config.json";


