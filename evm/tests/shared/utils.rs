use std::time::Duration;
use {
    crate::shared::genesis_private_key,
    ethereum_types::H160,
    ethereum_types::H256,
    ethers::{
        contract::{ContractFactory, ContractInstance},
        core::types::TransactionRequest,
        middleware::SignerMiddleware,
        providers::{Http, Middleware, Provider},
    },
    ethers_core::{
        abi::Abi,
        k256::ecdsa::SigningKey,
        rand::thread_rng,
        types::TransactionReceipt,
        types::{transaction::eip2718::TypedTransaction, Bytes, Eip1559TransactionRequest, U256},
    },
    ethers_signers::{LocalWallet, Signer, Wallet},
    std::future::Future,
    std::panic::{catch_unwind, AssertUnwindSafe},
    std::time::Instant,
    std::{env, sync::Arc},
};



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
pub fn return_current_provider(
    provider: &str,
    geth: Provider<Http>,
    proxy: Provider<Http>,
) -> Provider<Http> {
    match provider {
        "geth" => geth,
        "proxy" => proxy,
        _ => unimplemented!("Provider not supported"),
    }
}

fn run_on_network(network: &str) -> bool {
    let geth_url = env::var("GETH_URL").expect("GETH_URL environment variable not set");
    let proxy_url = env::var("PROXY_URL").expect("PROXY_URL environment variable not set");
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
pub async fn initial_setup() -> (
    Provider<Http>,
    Provider<Http>,
    Wallet<SigningKey>,
    Wallet<SigningKey>,
) {
    let start = Instant::now();
    println!("\n");
    let geth_url = env::var("GETH_URL").expect("GETH_URL environment variable not set");
    let proxy_url = env::var("PROXY_URL").expect("PROXY_URL environment variable not set");
    // default polling interval for event filters and pending transactions (default: 7 seconds)
    let pooling_interval = if run_on_devnet() {
        Duration::from_secs(4) // Default on Devnet
    } else if run_on_testnet() {
        Duration::from_secs(12) // Default on Testnet
    } else {
        Duration::from_secs(4) // Default on CI
    };

    // println!("GETH URL: {}", geth_url);
    // println!("PROXY URL: {}", proxy_url);
    let geth = Provider::<Http>::try_from(geth_url)
        .expect("Failed to create geth.")
        .interval(pooling_interval);
    let proxy = Provider::<Http>::try_from(proxy_url)
        .expect("Failed to create geth.")
        .interval(pooling_interval);
    println!("Default pooling interval: {:?}", pooling_interval);
    let private_key = genesis_private_key();
    let sender_wallet: LocalWallet = private_key.parse().expect("Failed to parse private key");
    let chain_id = proxy.get_chainid().await.unwrap().as_u64();
    let sender = sender_wallet.with_chain_id(chain_id);
    let receiver_wallet: LocalWallet = LocalWallet::new(&mut thread_rng());
    let receiver = receiver_wallet.with_chain_id(chain_id);
    println!("[ {:.2}s ] - Initial setup", start.elapsed().as_secs_f64());
    return (geth, proxy, sender, receiver);
}

#[allow(dead_code)]
pub async fn deploy_contract(
    contract: &str,
    provider: Provider<Http>,
    sender: Wallet<SigningKey>,
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
    let abi_path = format!("/opt/solidity/{}.abi", contract);
    let bytecode_path = format!("/opt/solidity/{}.bin", contract);
    let abi_string = std::fs::read_to_string(abi_path).expect("Failed to read ABI file");
    let abi: Abi = serde_json::from_str(&abi_string).expect("Failed to parse ABI");
    let bytecode_string =
        std::fs::read_to_string(bytecode_path).expect("Failed to read bytecode file");
    let bytecode: Bytes = hex::decode(bytecode_string.trim())
        .expect("Failed to decode bytecode")
        .into();

    let client = SignerMiddleware::new(provider, sender);
    let client = std::sync::Arc::new(client);
    let factory = ContractFactory::new(abi.clone(), bytecode, client);
    let contract = factory
        .deploy(())?
        .confirmations(1usize)
        .send_with_receipt()
        .await?;
    println!(
        "[ {:.2}s ] - Contract deployment: {}",
        start.elapsed().as_secs_f64(),
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

    while attempts < 3 {
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

// proxy responce is equal to geth responce
#[allow(dead_code)]
pub async fn check_state(proxy: &Provider<Http>, geth: &Provider<Http>, addresses: Vec<H160>) {
    let start = Instant::now();
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
    }
    println!("[ {:.2}s ] - Check state", start.elapsed().as_secs_f64());
}

#[allow(dead_code)]
pub async fn check_recipt(
    proxy: &Provider<Http>,
    geth: &Provider<Http>,
    recipt: &TransactionReceipt,
) {
    let start = Instant::now();
    let recipt_proxy = proxy
        .get_transaction_receipt(recipt.transaction_hash)
        .await
        .unwrap()
        .unwrap();
    let recipt_geth = geth
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
