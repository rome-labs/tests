use crate::shared::config::{load_config, Config};
use crate::shared::{
    client_config, CLIENT_CONFIG_PATH, PROXY_CONFIG_PATH, RECEIVER_PK, RHEA_CONFIG_PATH,
};
use solana_program::pubkey::Pubkey;
use std::process::Command;
use std::time::Duration;

use {
    crate::shared::genesis_private_key,
    ethereum_types::H160,
    ethereum_types::H256,
    ethers::{
        contract::{Contract, ContractFactory, ContractInstance},
        core::types::TransactionRequest,
        middleware::SignerMiddleware,
        providers::{Http, Middleware, Provider},
    },
    ethers_core::{
        abi::Abi,
        abi::AbiParser,
        k256::ecdsa::SigningKey,
        rand::thread_rng,
        types::TransactionReceipt,
        types::{transaction::eip2718::TypedTransaction, Bytes, Eip1559TransactionRequest, U256},
    },
    ethers_signers::{LocalWallet, Signer, Wallet},
    std::future::Future,
    std::panic::{catch_unwind, AssertUnwindSafe},
    std::time::Instant,
    std::{env, str::FromStr, sync::Arc},
};

#[derive(
    Debug, Clone, Default, serde::Deserialize, serde::Serialize, ethers::contract::EthAbiType,
)]
pub struct AccountBase58 {
    pub mint: String,
    pub owner: String,
    pub amount: u64,
    pub delegate: String,
    pub state: u8,
    pub is_native: bool,
    pub native_value: u64,
    pub delegated_amount: u64,
    pub close_authority: String,
}

#[allow(dead_code)]
fn get_rhea_config_path() -> String {
    env::var("RHEA_CONFIG").unwrap_or_else(|_| RHEA_CONFIG_PATH.to_string())
}

#[allow(dead_code)]
fn get_proxy_config_path() -> String {
    env::var("PROXY_CONFIG").unwrap_or_else(|_| PROXY_CONFIG_PATH.to_string())
}

#[allow(dead_code)]
pub fn get_default_config_path(provider: &str) -> String {
    match provider {
        "geth" => get_rhea_config_path(),
        "proxy" => get_proxy_config_path(),
        _ => unimplemented!("Provider not supported"),
    }
}

#[allow(dead_code)]
pub fn get_fee_addresses(provider: &str) -> Vec<H160> {
    let config = client_config(get_default_config_path(provider).as_str());
    let mut fee_addresses = Vec::new();
    for payer in &config.payers {
        if let Some(fees) = payer.fee_recipients() {
            for &addr in fees {
                fee_addresses.push(addr);
            }
        }
    }
    fee_addresses
}

#[allow(dead_code)]
pub fn prepare_tx(
    tx_type: &str,
    airdrop_amount: U256,
    nonce: U256,
    ethereum_receiver_address: H160,
    user: LocalWallet,
    chain_id: u64,
) -> TypedTransaction {
    match tx_type {
        "legacy" => {
            let tx = TransactionRequest::new()
                .to(ethereum_receiver_address)
                .value(airdrop_amount);
            tx.into()
        }
        "eip1559" => {
            let tx = Eip1559TransactionRequest {
                from: Some(user.address()),
                to: Some(ethereum_receiver_address.into()),
                data: None,
                nonce: Some(nonce.into()),
                chain_id: Some(chain_id.into()),
                value: Some(airdrop_amount),
                ..Default::default()
            };
            tx.into()
        }
        _ => unimplemented!("Transaction type not supported"),
    }
}

#[allow(dead_code)]
pub fn return_current_provider(provider: &str) -> Provider<Http> {
    match provider {
        "geth" => get_provider(get_geth_url()),
        "proxy" => get_provider(get_proxy_url()),
        _ => unimplemented!("Provider not supported"),
    }
}

#[allow(dead_code)]
fn run_on_network(network: &str) -> bool {
    let geth_url = get_geth_url();
    let proxy_url = get_proxy_url();
    geth_url.contains(network) || proxy_url.contains(network)
}

#[allow(dead_code)]
pub fn run_on_testnet() -> bool {
    run_on_network("barnet")
}

#[allow(dead_code)]
pub fn run_on_devnet() -> bool {
    run_on_network("foonet")
}

#[allow(dead_code)]
pub fn get_sender_wallet() -> Wallet<SigningKey> {
    let private_key = genesis_private_key();
    let sender_wallet: LocalWallet = private_key.parse().expect("Failed to parse private key");
    let chain_id = get_chain_id();
    sender_wallet.with_chain_id(chain_id)
}

#[allow(dead_code)]
pub fn get_receiver_wallet() -> Wallet<SigningKey> {
    let private_key = RECEIVER_PK;
    let receiver_wallet: LocalWallet = private_key.parse().expect("Failed to parse private key");
    let chain_id = get_chain_id();
    receiver_wallet.with_chain_id(chain_id)
}

#[allow(dead_code)]
pub fn get_random_wallet() -> Wallet<SigningKey> {
    let mut rng = thread_rng();
    let random_wallet: LocalWallet = LocalWallet::new(&mut rng);
    let chain_id = get_chain_id();
    random_wallet.with_chain_id(chain_id)
}

#[allow(dead_code)]
pub fn get_chain_id() -> u64 {
    let config: Config = load_config(CLIENT_CONFIG_PATH).expect("Failed to load configuration");
    config.chain_id
}

#[allow(dead_code)]
pub fn get_program_id() -> Pubkey {
    let config: Config = load_config(CLIENT_CONFIG_PATH).expect("Failed to load configuration");
    Pubkey::from_str(&config.program_id).unwrap()
}

#[allow(dead_code)]
pub fn get_solana_key(address: H160) -> Pubkey {
    let chain_id = get_chain_id().to_le_bytes().to_vec();
    let program_id = get_program_id();
    let (solana_key, _seed) = balance_key(chain_id, &program_id, &address);
    solana_key
}

#[allow(dead_code)]
fn get_pooling_interval() -> Duration {
    // default polling interval for event filters and pending transactions (default: 7 seconds) -e POOLING_INTERVAL=$POOLING_INTERVAL
    let pooling_interval = env::var("POOLING_INTERVAL")
        .ok()
        .and_then(|s| u64::from_str(&s).ok())
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(7));
    pooling_interval
}

#[allow(dead_code)]
pub fn get_provider(url: String) -> Provider<Http> {
    let pooling_interval = get_pooling_interval();
    let provider = Provider::<Http>::try_from(url)
        .expect("Failed to create provider.")
        .interval(pooling_interval);
    provider
}

#[allow(dead_code)]
fn get_geth_url() -> String {
    env::var("GETH_URL").unwrap_or("http://localhost:8545".to_string())
}

#[allow(dead_code)]
fn get_proxy_url() -> String {
    env::var("PROXY_URL").unwrap_or("http://localhost:9090".to_string())
}

#[allow(dead_code)]
pub async fn get_account_state(
    contract_name: &str,
    contract_address: H160,
    spl_account: Pubkey,
    provider_name: &str,
    sender: Wallet<SigningKey>,
) -> Result<AccountBase58, Box<dyn std::error::Error>> {
    let abi = get_abi(contract_name);

    let provider = return_current_provider(provider_name);
    let client = Arc::new(SignerMiddleware::new(provider.clone(), sender));
    let contract = Contract::new(contract_address, abi, client.clone());
    let result: AccountBase58 = contract
        .method::<_, AccountBase58>("account_state", spl_account.to_string())?
        .call()
        .await?;
    Ok(result)
}

#[allow(dead_code)]
pub async fn initial_setup() -> (
    Provider<Http>,
    Provider<Http>,
    Wallet<SigningKey>,
    Wallet<SigningKey>,
) {
    let geth = get_provider(get_geth_url());
    let proxy = get_provider(get_proxy_url());
    let sender = get_random_wallet();
    let _ = airdrop_to_address(sender.address(), U256::exp10(18), "proxy").await;
    let receiver_address = get_receiver_wallet();
    return (geth, proxy, sender, receiver_address);
}

#[allow(dead_code)]
pub fn get_abi(contract: &str) -> Abi {
    let abi_path = format!("/opt/solidity/{}.abi", contract);
    let abi_string = std::fs::read_to_string(abi_path).expect("Failed to read ABI file");
    let abi: Abi = serde_json::from_str(&abi_string).expect("Failed to parse ABI");
    abi
}

#[allow(dead_code)]
pub fn get_bin(contract: &str) -> Bytes {
    let bytecode_path = format!("/opt/solidity/{}.bin", contract);
    let bytecode_string =
        std::fs::read_to_string(bytecode_path).expect("Failed to read bytecode file");
    let bytecode: Bytes = hex::decode(bytecode_string.trim())
        .expect("Failed to decode bytecode")
        .into();
    bytecode
}

#[allow(dead_code)]
pub async fn deploy_contract(
    contract_name: &str,
    provider_name: &str,
    sender_wallet: &Wallet<SigningKey>,
) -> Result<
    (
        ContractInstance<
            Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
            SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
        >,
        TransactionReceipt,
    ),
    Box<dyn std::error::Error>,
> {
    let start = Instant::now();
    // Preparation
    let abi: Abi = get_abi(contract_name);
    let bytecode: Bytes = get_bin(contract_name);
    let provider = return_current_provider(provider_name);
    let client = SignerMiddleware::new(provider, sender_wallet.clone());
    let client = std::sync::Arc::new(client);
    let factory = ContractFactory::new(abi.clone(), bytecode, client);
    let contract = factory
        .deploy(())?
        .confirmations(1usize)
        .send_with_receipt()
        .await?;
    println!(
        "[ {:.2}s ] - Contract {} deployment: {}",
        start.elapsed().as_secs_f64(),
        contract_name,
        contract.0.address().to_string()
    );
    Ok(contract)
}

#[allow(dead_code)]
pub async fn retry<F, Fut, T, E>(mut f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempts = 0;
    while attempts < 3 {
        println!("Attempt {}...", attempts + 1);
        match f().await {
            Ok(result) => {
                println!("Attempt {} succeeded.", attempts + 1);
                return Ok(result);
            }
            Err(_) if attempts < 2 => {
                println!("Attempt {} failed. Retrying...", attempts + 1);
                attempts += 1;
            }
            Err(e) => {
                println!("Attempt {} failed. No more retries.", attempts + 1);
                return Err(e);
            }
        }
    }
    let final_err = f().await.err().unwrap();
    println!("All attempts failed.");
    Err(final_err)
}

#[allow(dead_code)]
pub async fn retry_panic<F, Fut, T>(mut f: F) -> Result<(), ()>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = T>,
{
    let mut attempts = 0;

    while attempts < 6 {
        println!("Attempt {}...", attempts + 1);

        let result = catch_unwind(AssertUnwindSafe(|| futures::executor::block_on(f())));

        match result {
            Err(_) => {
                println!(
                    "Test panicked on attempt {}. Expected behavior, stopping retries.",
                    attempts + 1
                );
                panic!("The test panicked as expected");
            }
            Ok(_) if attempts < 2 => {
                println!("Attempt {} did NOT panic. Retrying...", attempts + 1);
                attempts += 1;
            }
            Ok(_) => {
                println!("All attempts completed without panic. Test FAILED (expected panic).");
                return Ok(()); // No panic after 3 tries => test should fail
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn check_storage(
    provider: Provider<Http>,
    proxy: &Provider<Http>,
    geth: &Provider<Http>,
    address: H160,
    slots: Option<Vec<u64>>,
    values: Option<Vec<u64>>,
) {
    let start = Instant::now();
    if let (Some(slots), Some(values)) = (slots.as_ref(), values.as_ref()) {
        for (slot, value) in slots.iter().zip(values.iter()) {
            assert_eq!(
                provider
                    .get_storage_at(address, H256::from_low_u64_be(*slot), None)
                    .await
                    .unwrap()
                    .to_low_u64_be(),
                *value,
                "Values does not match | actual != expected"
            );
            assert_eq!(
                proxy
                    .get_storage_at(address, H256::from_low_u64_be(*slot), None)
                    .await
                    .unwrap(),
                geth.get_storage_at(address, H256::from_low_u64_be(*slot), None)
                    .await
                    .unwrap(),
                "Storages for proxy and geth should match"
            );
        }
    }
    println!("[ {:.2}s ] - Check storage", start.elapsed().as_secs_f64());
}

#[allow(dead_code)]
pub async fn sum_fee_balances(provider_name: &str) -> U256 {
    let provider = return_current_provider(provider_name);
    let fee_addresses = get_fee_addresses(provider_name);
    let mut sum = U256::zero();
    for addr in &fee_addresses {
        sum += provider.get_balance(*addr, None).await.unwrap();
    }
    sum
}


// proxy responce is equal to geth responce
#[allow(dead_code)]
pub async fn check_state(addresses: Vec<H160>) {
    let start = Instant::now();
    let proxy = get_provider(get_proxy_url());
    let geth = get_provider(get_geth_url());
    let proxy_sum = sum_fee_balances("proxy").await;
    let geth_sum = sum_fee_balances("geth").await;
    for address in addresses {
        // Check Nonce
        assert_eq!(
            proxy.get_transaction_count(address, None).await.unwrap(),
            geth.get_transaction_count(address, None).await.unwrap(),
            "Nonces should match between providers: Proxy == Geth"
        );
        // Check Balance
        assert_eq!(
            proxy.get_balance(address, None).await.unwrap(),
            geth.get_balance(address, None).await.unwrap(),
            "Balances should match between providers: Proxy == Geth"
        );
        // Check Code
        assert_eq!(
            proxy.get_code(address, None).await.unwrap(),
            geth.get_code(address, None).await.unwrap(),
            "Code should match between providers: Proxy == Geth"
        );
        // Check fee Balance
        assert_eq!(
            proxy_sum, geth_sum,
            "Fee balances should match: Proxy == Geth"
        );
    }
    println!("[ {:.2}s ] - Check state", start.elapsed().as_secs_f64());
}

#[allow(dead_code)]
pub async fn check_recipt(recipt: &TransactionReceipt) {
    let start = Instant::now();
    let recipt_proxy = get_provider(get_proxy_url())
        .get_transaction_receipt(recipt.transaction_hash)
        .await
        .unwrap()
        .unwrap();
    let recipt_geth = get_provider(get_geth_url())
        .get_transaction_receipt(recipt.transaction_hash)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        recipt_proxy.transaction_hash, recipt_geth.transaction_hash,
        "Transaction hashes should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.block_number, recipt_geth.block_number,
        "Block numbers should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.block_hash, recipt_geth.block_hash,
        "Block hashes should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.from, recipt_geth.from,
        "From addresses should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.to, recipt_geth.to,
        "To addresses should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.logs, recipt_geth.logs,
        "Logs should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.logs_bloom, recipt_geth.logs_bloom,
        "Logs bloom should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.status, recipt_geth.status,
        "Status should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.cumulative_gas_used, recipt_geth.cumulative_gas_used,
        "Cumulative gas used should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.gas_used, recipt_geth.gas_used,
        "Gas used should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.transaction_index, recipt_geth.transaction_index,
        "Transaction indexes should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.logs_bloom, recipt_geth.logs_bloom,
        "Logs bloom should match between providers: Proxy == Geth"
    );
    assert_eq!(
        recipt_proxy.logs, recipt_geth.logs,
        "Logs should match between providers: Proxy == Geth"
    );
    println!("[ {:.2}s ] - Check recipt", start.elapsed().as_secs_f64());
}

#[allow(dead_code)]
pub fn balance_key(chain: Vec<u8>, program_id: &Pubkey, address: &H160) -> (Pubkey, Vec<Vec<u8>>) {
    const ACCOUNT_SEED: &[u8] = b"ACCOUN_SEED";
    let mut seed = vec![
        chain.clone(),
        ACCOUNT_SEED.to_vec(),
        address.as_bytes().to_vec(),
    ];

    // Derive the PDA
    let (key, bump_seed) = Pubkey::find_program_address(
        &seed.iter().map(|s| s.as_slice()).collect::<Vec<&[u8]>>(),
        program_id,
    );

    // Add the bump seed to the seed vector
    seed.push(vec![bump_seed]);

    (key, seed)
}

#[allow(dead_code)]
pub fn get_spl(
    // get_associated_token_address_and_bump_seed_internal
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    program_id: &Pubkey,
    token_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &wallet_address.to_bytes(),
            &token_program_id.to_bytes(),
            &token_mint_address.to_bytes(),
        ],
        program_id,
    )
}

#[allow(dead_code)]
pub async fn create_spl_account(
    provider: &str,
    solana_key: Pubkey,
    mint_address: Pubkey,
    contract_address: H160,
    sender: Wallet<SigningKey>,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = return_current_provider(provider);
    let client = Arc::new(SignerMiddleware::new(provider.clone(), sender));

    let second_abi = AbiParser::default().parse(&[
        "function create_associated_token_account(string user, string mint) returns (string)",
    ]);

    let contract = Contract::new(contract_address, second_abi.unwrap(), client.clone());

    let receipt = contract
        .method::<_, ()>(
            "create_associated_token_account",
            (solana_key.to_string(), mint_address.to_string()),
        )?
        .send()
        .await?
        .await?;
    if let Some(receipt) = receipt {
        println!(
            "[ Info: ] - SPL account creation transaction hash: {:?}",
            receipt.transaction_hash
        );
    } else {
        println!("[ Error ] - Receipt is None, transaction hash unavailable.");
    }
    Ok(())
}

#[allow(dead_code)]
pub fn mint_to(mint_address: &str, amount: &str, sender: Pubkey) {
    let rpc_url = if get_proxy_url().contains("localhost") || get_geth_url().contains("localhost") {
        "http://localhost:8899"
    } else {
        "http://solana:8899"
    };
    // Execute the `spl-token mint` command
    let airdrop_output = Command::new("/opt/bin/solana")
        .arg("airdrop")
        .arg("-u")
        .arg(rpc_url)
        .arg("100")
        .arg("-k")
        .arg("/opt/ci/keys/mint-authority.json")
        .output()
        .expect("Failed to execute solana airdrop command");

    // Check the output of the airdrop command
    if airdrop_output.status.success() {
        println!("[ Info: ] - Airdrop successful!");
        // println!("{}", String::from_utf8_lossy(&airdrop_output.stdout));
    } else {
        eprintln!("[ Error ] - Airdrop failed!");
        eprintln!("{}", String::from_utf8_lossy(&airdrop_output.stderr));
    }

    let output = Command::new("/opt/bin/spl-token")
        .arg("mint")
        .arg(mint_address)
        .arg(amount)
        .arg(sender.to_string())
        .arg("--mint-authority")
        .arg("/opt/ci/keys/mint-authority.json")
        .arg("--fee-payer")
        .arg("/opt/ci/keys/mint-authority.json")
        .arg("-u")
        .arg(rpc_url)
        .output()
        .expect("Failed to execute spl-token mint command");

    // Check the output
    if output.status.success() {
        println!("[ Info: ] - Minting successful!");
        // println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("[ Error ] - Minting failed!");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[allow(dead_code)]
pub async fn transfer(
    contract_address: H160,
    from: Pubkey,
    to: Pubkey,
    amount: u64,
    sender: Wallet<SigningKey>,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = return_current_provider("proxy");
    let client = Arc::new(SignerMiddleware::new(provider.clone(), sender));

    // Deploy contract
    let contract_spl_name = "SplHolderT";
    let abi = get_abi(contract_spl_name);

    let contract = Contract::new(contract_address, abi, client.clone());
    let _ = contract
        .method::<_, H256>(
            "transfer",
            (
                H256::from_slice(&from.to_bytes()),
                H256::from_slice(&to.to_bytes()),
                amount,
            ),
        )?
        .send()
        .await?;
    Ok(())
}

#[allow(dead_code)]
pub fn solana_balance(address: Pubkey) -> U256 {
    let rpc_url = if get_proxy_url().contains("localhost") || get_geth_url().contains("localhost") {
        "http://localhost:8899"
    } else {
        "http://solana:8899"
    };

    let output = Command::new("/opt/bin/solana")
        .arg("balance")
        .arg(address.to_string())
        .arg("-u")
        .arg(rpc_url)
        .output()
        .expect("Failed to execute solana balance command");

    if output.status.success() {
        let balance_str = String::from_utf8_lossy(&output.stdout);
        let balance_sol: f64 = balance_str
            .trim()
            .strip_suffix(" SOL")
            .expect("Failed to strip SOL suffix")
            .parse()
            .expect("Failed to parse balance as f64");
        let balance_lamports = U256::from((balance_sol * 1_000_000_000.0) as u64);
        return balance_lamports;
    } else {
        eprintln!("[ Error ] - Balance check failed!");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    U256::from(0)
}

#[allow(dead_code)]
pub async fn airdrop_to_address(
    address: H160,
    amount: U256,
    provider_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = return_current_provider(provider_name);
    let sender = get_sender_wallet();
    let client = SignerMiddleware::new(provider.clone(), sender.clone());

    let tx_type: &str = "legacy";
    let tx = prepare_tx(
        tx_type,
        amount,
        provider
            .get_transaction_count(sender.address(), None)
            .await
            .unwrap(), // Nonce
        address,
        sender.clone(),
        get_chain_id(),
    );

    let pending_tx = client.send_transaction(tx, None).await?;
    let _receipt = pending_tx
        .confirmations(1usize)
        .await
        .map_err(|_| "[ Info: ] - Transaction dropped from mempool")?;
    Ok(())
}

#[allow(dead_code)]
pub async fn transfer_tx(
    tx_types: Vec<&str>,
    airdrop_amount: U256,
    provider_name: &str,
    sender: Wallet<SigningKey>,
    receiver_address: H160,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = return_current_provider(provider_name);
    let chain_id: u64 = provider.get_chainid().await.unwrap().as_u64();
    let client = SignerMiddleware::new(provider.clone(), sender.clone());
    
    for tx_type in tx_types {
        let start = Instant::now();
        let tx = prepare_tx(
            tx_type,
            U256::from(airdrop_amount),
            provider
                .get_transaction_count(sender.address(), None)
                .await
                .unwrap(), // Nonce
            receiver_address,
            sender.clone(),
            chain_id,
        );
        println!("[ {:.2}s ] - Tx preparation", start.elapsed().as_secs_f64());

        let start = Instant::now();
        let pending_tx = client.send_transaction(tx, None).await?;
        let receipt = pending_tx
            .confirmations(1usize)
            .await?
            .ok_or("[ Info: ] - Transaction dropped from mempool")?;
        println!(
            "[ {:.2}s ] - Tx send: {}",
            start.elapsed().as_secs_f64(),
            receipt.transaction_hash
        );

        println!("[ Info: ] - Checking state after {}", tx_type);
        check_state(vec![sender.address(), receiver_address]).await;
    }
    Ok(())
}