mod shared;

use {
    ethereum_types::H256, 
    ethers::{providers::{Http, Middleware, Provider}, 
    types::TransactionReceipt}, 
    ethers_core::types::{
        transaction::eip2718::TypedTransaction, Bytes, Eip1559TransactionRequest, NameOrAddress,
        U256,
    }, 
    ethers_signers::Signer, rstest::rstest, serial_test::serial, 
    shared::{
        client::Client, fixture::client, tx::{abi, do_tx, method_id}, wallet, CONTRACTS
    }, 
    std::{ops::{Mul, Sub}, time::Duration}, 
    tokio::time::{sleep, timeout},
};

#[rstest]
#[serial]
#[ignore]
async fn state_gas_transfer_native_token_transfer(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from("http://geth:8545")
        .expect("Failed to create provider.");

    const AIRDROP_AMOUNT: u64 = 10000000000000;
    let from = client.rollup_owner_wallet.address();
    println!("owner_wallet = {:?}", from);

    let evm_user = wallet();
    println!("evm_user = {:?}", evm_user.address());

    let gas_recipient = wallet();
    println!("gas_recipient = {:?}", gas_recipient.address());

    // client.reg_gas_recipient(gas_recipient.address().clone(), payer).await.unwrap();

    // Check pre-conditions
    println!("Pre-checks...");
    assert_eq!(client.get_balance(evm_user.address()).unwrap(), U256::zero());
    assert_ne!(client.get_balance(from).unwrap(), U256::zero());
    let owner_pre_balance = client.get_balance(from).unwrap();
    assert_eq!(client.get_balance(from.clone()).unwrap(), provider.get_balance(from.clone(), None).await.unwrap());
    assert_eq!(client.get_balance(gas_recipient.address()).unwrap(), provider.get_balance(gas_recipient.address(), None).await.unwrap());
    assert_eq!(client.get_balance(evm_user.address()).unwrap(), provider.get_balance(evm_user.address(), None).await.unwrap());
    println!("Pre-checks OK");

    // Prepare and send native token transfer
    let nonce = client.transaction_count(from).unwrap().as_u64();
    let mut eip1559 = Eip1559TransactionRequest {
        from: Some(from),
        to: Some(NameOrAddress::Address(evm_user.address())),
        data: None,
        nonce: Some(nonce.into()),
        chain_id: Some(client.chain_id().into()),
        value: Some(U256::from(AIRDROP_AMOUNT)),
        ..Default::default()
    };

    let gas_estimate = client.estimate_gas(&eip1559.clone().into()).unwrap();
    println!("gas_estimate = {:?}", gas_estimate);
    let gas_price = client.gas_price().unwrap();
    println!("gas_price = {:?}", gas_price);
    eip1559.gas = Some(gas_estimate);
    eip1559.max_priority_fee_per_gas = Some(gas_price);
    let tx = TypedTransaction::Eip1559(eip1559);
    let sig = client.rollup_owner_wallet.sign_transaction_sync(&tx).unwrap();
    let rlp = tx.rlp_signed(&sig).to_vec();
    let hash = tx.hash(&sig);
    println!("Transaction hash: {:?}", hash);
    client.send_transaction(Bytes::from(rlp)).await.unwrap();
    sleep(Duration::from_secs(5)).await;

    // check post-conditions
    println!("Post-checks...");
    assert_eq!(client.get_balance(evm_user.address()).unwrap(), U256::from(AIRDROP_AMOUNT));
    let exp_owner_post_balance = owner_pre_balance
        .sub(gas_estimate.mul(gas_price))
        .sub(AIRDROP_AMOUNT);
    assert_eq!(client.get_balance(from.clone()).unwrap(), exp_owner_post_balance);
    assert_eq!(client.get_balance(from.clone()).unwrap(), provider.get_balance(from.clone(), None).await.unwrap());
    assert_eq!(client.get_balance(gas_recipient.address()).unwrap(), provider.get_balance(gas_recipient.address(), None).await.unwrap());
    assert_eq!(client.get_balance(evm_user.address()).unwrap(), provider.get_balance(evm_user.address(), None).await.unwrap());
    println!("Post-checks OK");

    Ok(())
}

#[rstest(contract, case::storage("TouchStorage"))]
#[serial]
#[ignore]
async fn state_gas_transfer_contract_deployment(client: &Client, contract: String) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from("http://geth:8545")
        .expect("Failed to create provider.");
    let from = client.rollup_owner_wallet.address();

    println!("deployer_wallet = {:?}", from);
    let gas_recipient = wallet();
    println!("gas_recipient = {:?}", gas_recipient.address());
    // client.reg_gas_recipient(gas_recipient.address().clone()).await.unwrap();

    // Check pre-conditions
    let deployer_pre_balance = client.get_balance(from).unwrap();
    assert_ne!(client.get_balance(from).unwrap(), U256::zero());
    assert_eq!(client.get_balance(from.clone()).unwrap(), provider.get_balance(from.clone(), None).await.unwrap());
    assert_eq!(client.get_balance(gas_recipient.address()).unwrap(), provider.get_balance(gas_recipient.address(), None).await.unwrap());
    println!("deployer_pre_balance = {:?}", deployer_pre_balance);

    // Prepare and send contract deployment
    let path = format!("{}{}.binary", CONTRACTS, contract);
    let bin = std::fs::read(&path).unwrap();
    let nonce = client.transaction_count(from).unwrap().as_u64();
    let mut eip1559 = Eip1559TransactionRequest {
        from: Some(from),
        to: None,
        data: Some(Bytes::from(bin)),
        nonce: Some(nonce.into()),
        chain_id: Some(client.chain_id().into()),
        value: None,
        ..Default::default()
    };

    let gas_estimate = client.estimate_gas(&eip1559.clone().into()).unwrap();
    println!("gas_estimate = {:?}", gas_estimate);
    let gas_price = client.gas_price().unwrap();
    println!("gas_price = {:?}", gas_price);
    eip1559.gas = Some(gas_estimate);
    eip1559.max_priority_fee_per_gas = Some(gas_price);
    let tx = TypedTransaction::Eip1559(eip1559);
    let sig = client.rollup_owner_wallet.sign_transaction_sync(&tx).unwrap();
    let rlp = tx.rlp_signed(&sig).to_vec();
    client.send_transaction(Bytes::from(rlp)).await.unwrap();
    sleep(Duration::from_secs(5)).await;

    // check post-conditions
    let exp_deployer_post_balance = deployer_pre_balance.sub(gas_estimate.mul(gas_price));
    println!("exp_deployer_post_balance = {:?}", exp_deployer_post_balance);
    assert_eq!(client.get_balance(from).unwrap(), exp_deployer_post_balance);
    assert_eq!(client.get_balance(from.clone()).unwrap(), provider.get_balance(from.clone(), None).await.unwrap());
    assert_eq!(client.get_balance(gas_recipient.address()).unwrap(), provider.get_balance(gas_recipient.address(), None).await.unwrap());

    Ok(())
}

#[rstest]
#[serial]
#[ignore]
async fn state_gas_transfer_native_token_transfer_op_geth(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from("http://geth:8545")
        .expect("Failed to create provider.");

    const AIRDROP_AMOUNT: u64 = 10000000000000;
    // let _payer = client.get_rhea_sender();
    let from = client.rollup_owner_wallet.address();
    println!("owner_wallet = {:?}", from);

    let evm_user = wallet();
    println!("evm_user = {:?}", evm_user.address());

    let gas_recipient = wallet();
    println!("gas_recipient = {:?}", gas_recipient.address());

    // client.reg_gas_recipient(gas_recipient.address().clone(), payer).await.unwrap();

    // Check pre-conditions
    println!("Pre-checks...");
    assert_eq!(client.get_balance(evm_user.address()).unwrap(), U256::zero());
    assert_ne!(client.get_balance(from).unwrap(), U256::zero());
    let owner_pre_balance = client.get_balance(from).unwrap();
    assert_eq!(client.get_balance(from.clone()).unwrap(), provider.get_balance(from.clone(), None).await.unwrap());
    assert_eq!(client.get_balance(gas_recipient.address()).unwrap(), provider.get_balance(gas_recipient.address(), None).await.unwrap());
    assert_eq!(client.get_balance(evm_user.address()).unwrap(), provider.get_balance(evm_user.address(), None).await.unwrap());
    println!("Pre-checks OK");

    // Prepare and send native token transfer
    let nonce = client.transaction_count(from).unwrap().as_u64();
    let mut eip1559 = Eip1559TransactionRequest {
        from: Some(from),
        to: Some(NameOrAddress::Address(evm_user.address())),
        data: None,
        nonce: Some(nonce.into()),
        chain_id: Some(client.chain_id().into()),
        value: Some(U256::from(AIRDROP_AMOUNT)),
        ..Default::default()
    };

    let gas_estimate = client.estimate_gas(&eip1559.clone().into()).unwrap();
    println!("gas_estimate = {:?}", gas_estimate);
    let gas_price = client.gas_price().unwrap();
    println!("gas_price = {:?}", gas_price);
    eip1559.gas = Some(gas_estimate);
    eip1559.max_priority_fee_per_gas = Some(gas_price);
    eip1559.max_fee_per_gas = Some(gas_price.mul(2));
    let tx = TypedTransaction::Eip1559(eip1559);
    let sig = client.rollup_owner_wallet.sign_transaction_sync(&tx).unwrap();
    let rlp = tx.rlp_signed(&sig).to_vec();
    let hash = tx.hash(&sig);
    println!("Transaction hash: {:?}", hash);
    provider.send_raw_transaction(Bytes::from(rlp)).await.unwrap();
    let _tx_receipt = wait_for_transaction_receipt(&provider, hash, gas_estimate).await.unwrap();

    // check post-conditions
    println!("Post-checks...");
    assert_eq!(client.get_balance(evm_user.address()).unwrap(), U256::from(AIRDROP_AMOUNT));
    let exp_owner_post_balance = owner_pre_balance
        .sub(gas_estimate.mul(gas_price))
        .sub(AIRDROP_AMOUNT);
    assert_eq!(client.get_balance(from.clone()).unwrap(), exp_owner_post_balance);
    assert_eq!(client.get_balance(from.clone()).unwrap(), provider.get_balance(from.clone(), None).await.unwrap());
    assert_eq!(client.get_balance(gas_recipient.address()).unwrap(), provider.get_balance(gas_recipient.address(), None).await.unwrap());
    assert_eq!(client.get_balance(evm_user.address()).unwrap(), provider.get_balance(evm_user.address(), None).await.unwrap());
    println!("Post-checks OK");

    Ok(())
}

#[rstest(contract, case::storage("TouchStorage"))]
#[serial]
#[ignore]
async fn state_gas_transfer_contract_deployment_op_geth(client: &Client, contract: String) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from("http://geth:8545")
        .expect("Failed to create provider.");
    let from = client.rollup_owner_wallet.address();
    // let _payer = client.get_rhea_sender();

    println!("deployer_wallet = {:?}", from);
    let gas_recipient = wallet();
    println!("gas_recipient = {:?}", gas_recipient.address());
    
    // client.reg_gas_recipient(gas_recipient.address().clone(), payer).await.unwrap();
    let gas_recipient = gas_recipient.address();

    // Check pre-conditions
    let deployer_pre_balance = client.get_balance(from).unwrap();
    assert_ne!(client.get_balance(from).unwrap(), U256::zero());
    assert_eq!(client.get_balance(from.clone()).unwrap(), provider.get_balance(from.clone(), None).await.unwrap());
    assert_eq!(client.get_balance(gas_recipient).unwrap(), provider.get_balance(gas_recipient, None).await.unwrap());
    println!("deployer_pre_balance = {:?}", deployer_pre_balance);

    // Prepare and send contract deployment
    let path = format!("{}{}.binary", CONTRACTS, contract);
    let bin = std::fs::read(&path).unwrap();
    let nonce = client.transaction_count(from).unwrap().as_u64();
    let mut eip1559 = Eip1559TransactionRequest {
        from: Some(from),
        to: None,
        data: Some(Bytes::from(bin)),
        nonce: Some(nonce.into()),
        chain_id: Some(client.chain_id().into()),
        value: None,
        ..Default::default()
    };

    let gas_estimate = client.estimate_gas(&eip1559.clone().into()).unwrap();
    println!("gas_estimate = {:?}", gas_estimate);
    let gas_price = client.gas_price().unwrap();
    println!("gas_price = {:?}", gas_price);
    eip1559.gas = Some(gas_estimate);
    eip1559.max_priority_fee_per_gas = Some(gas_price);
    eip1559.max_fee_per_gas = Some(gas_price.mul(2));
    let tx = TypedTransaction::Eip1559(eip1559);
    let sig = client.rollup_owner_wallet.sign_transaction_sync(&tx).unwrap();
    let rlp = tx.rlp_signed(&sig).to_vec();
    let hash = tx.hash(&sig);
    provider.send_raw_transaction(Bytes::from(rlp)).await.unwrap();
    let _tx_receipt = wait_for_transaction_receipt(&provider, hash, gas_estimate).await.unwrap();

    // check post-conditions
    let exp_deployer_post_balance = deployer_pre_balance.sub(gas_estimate.mul(gas_price));
    println!("exp_deployer_post_balance = {:?}", exp_deployer_post_balance);
    assert_eq!(client.get_balance(from).unwrap(), exp_deployer_post_balance);
    assert_eq!(client.get_balance(from.clone()).unwrap(), provider.get_balance(from.clone(), None).await.unwrap());
    assert_eq!(client.get_balance(gas_recipient).unwrap(), provider.get_balance(gas_recipient, None).await.unwrap());

    Ok(())
}

#[rstest(
    contract,
    methods,
    tx_type,
    case::storage("IterativeOpGeth", vec!["iterative"], 2),
)]
#[serial]
#[ignore]
async fn state_gas_transfer_iterative_contract_deployment_op_geth(
    client: &Client,
    contract: String,
    methods: Vec<&str>,
    tx_type: u8
) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from("http://geth:8545")
        .expect("Failed to create provider.");
    let from = client.rollup_owner_wallet.address();
    // let _payer = client.get_rhea_sender();

    println!("deployer_wallet = {:?}", from);
    let gas_recipient = wallet();
    println!("gas_recipient = {:?}", gas_recipient.address());
    
    // client.reg_gas_recipient(gas_recipient.address().clone(), payer).await.unwrap();
    let gas_recipient = gas_recipient.address();
 
    // Check pre-conditions
    let deployer_pre_balance = client.get_balance(from).unwrap();
    assert_ne!(client.get_balance(from).unwrap(), U256::zero());
    assert_eq!(client.get_balance(from.clone()).unwrap(), provider.get_balance(from.clone(), None).await.unwrap());
    assert_eq!(client.get_balance(gas_recipient).unwrap(), provider.get_balance(gas_recipient, None).await.unwrap());
    println!("deployer_pre_balance = {:?}", deployer_pre_balance);

    // Prepare and send contract deployment
    let abi = abi(&format!("{}{}.abi", CONTRACTS, contract));
    let path = format!("{}{}.binary", CONTRACTS, contract);
    let bin = std::fs::read(&path).unwrap();
    let nonce = client.transaction_count(from).unwrap().as_u64();
    let mut eip1559 = Eip1559TransactionRequest {
        from: Some(from),
        to: None,
        data: Some(Bytes::from(bin)),
        nonce: Some(nonce.into()),
        chain_id: Some(client.chain_id().into()),
        value: None,
        ..Default::default()
    };

    let gas_estimate = client.estimate_gas(&eip1559.clone().into()).unwrap();
    println!("gas_estimate = {:?}", gas_estimate);
    let gas_price = client.gas_price().unwrap();
    println!("gas_price = {:?}", gas_price);
    eip1559.gas = Some(gas_estimate);
    eip1559.max_priority_fee_per_gas = Some(gas_price);
    eip1559.max_fee_per_gas = Some(gas_price.mul(2));
    let tx = TypedTransaction::Eip1559(eip1559);
    let sig = client.rollup_owner_wallet.sign_transaction_sync(&tx).unwrap();
    let hash = tx.hash(&sig);
    let rlp = tx.rlp_signed(&sig).to_vec();
    provider.send_raw_transaction(Bytes::from(rlp)).await.unwrap();
    let tx_receipt = wait_for_transaction_receipt(&provider, hash, gas_estimate).await.unwrap();
    let contract_address = tx_receipt.contract_address;

    // check post-conditions
    let exp_deployer_post_balance = deployer_pre_balance.sub(gas_estimate.mul(gas_price));
    println!("exp_deployer_post_balance = {:?}", exp_deployer_post_balance);
    assert_eq!(client.get_balance(from).unwrap(), exp_deployer_post_balance);
    assert_eq!(client.get_balance(from.clone()).unwrap(), provider.get_balance(from.clone(), None).await.unwrap());
    assert_eq!(client.get_balance(gas_recipient).unwrap(), provider.get_balance(gas_recipient, None).await.unwrap());

    for method in methods {
        let tx = do_tx(&client, contract_address, method_id(&abi, method), &client.rollup_owner_wallet, 0, tx_type);
        let estimate_gas = tx.gas().unwrap().clone();
        let gas_price = client.gas_price().unwrap();
    
        // check and compute balance pre-method submission
        let before = client.get_balance(from).unwrap();
        let before_op_geth = provider.get_balance(from, None).await.unwrap();
        assert_eq!(before, before_op_geth);
        let deployer_post_balance = client.get_balance(from).unwrap();
        let exp_deployer_post_balance = deployer_post_balance.sub(estimate_gas.mul(gas_price));

        // submit method call
        let sig = client.rollup_owner_wallet.sign_transaction_sync(&tx).unwrap();
        let hash = tx.hash(&sig);
        let rlp = tx.rlp_signed(&sig);
        provider.send_raw_transaction(Bytes::from(rlp)).await.unwrap();
        let _tx_receipt = wait_for_transaction_receipt(&provider, hash, estimate_gas).await.unwrap();
        
        // check balance post-method submission
        let after: U256 = client.get_balance(from).unwrap();
        assert_eq!(after, exp_deployer_post_balance);
        let after_op_geth = provider.get_balance(from, None).await.unwrap();
        assert_eq!(after, after_op_geth);
        
        let gas_transfer = before.checked_sub(after).unwrap();
        assert!(gas_transfer >= 0.into());
        assert!(gas_transfer <= estimate_gas);
    }

    Ok(())
}

async fn wait_for_transaction_receipt(
    provider: &Provider::<Http>, 
    hash: H256,          
    estimated_gas: U256, 
) -> Result<TransactionReceipt, &'static str> {
    timeout(Duration::from_secs(30), async {
        loop {
            match provider.get_transaction_receipt(hash).await {
                Ok(Some(receipt)) => {
                    assert_eq!(receipt.gas_used.unwrap(), estimated_gas);
                    return Ok(receipt);
                }
                Ok(None) => {
                    println!("Transaction not found or still pending, waiting for the next confirmation...");
                    sleep(Duration::from_millis(150)).await;
                }
                Err(_) => return Err("Failed to retrieve transaction receipt"),
            }
        }
    })
    .await
    .map_err(|_| "Timeout waiting for transaction receipt")?
}