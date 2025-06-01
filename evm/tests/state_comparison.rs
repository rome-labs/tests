mod shared;

use crate::shared::utils::{
    airdrop_to_address, check_recipt, check_state, check_storage, deploy_contract,
    get_receiver_wallet, initial_setup, return_current_provider, solana_balance,
    sum_fee_balances, transfer_tx,
};
use shared::{test_account, utils::get_random_wallet, WITHDRAWAL_ADDRESS};
use std::time::Instant;
use {
    ethereum_types::H256,
    ethers::{middleware::SignerMiddleware, providers::Middleware},
    ethers_core::types::{Bytes, TransactionRequest, H160, U256},
    ethers_signers::Signer,
    rstest::*,
    serial_test::serial,
    std::str::FromStr,
    std::sync::Arc,
};

#[rstest(
    provider_name,
    airdrop_amount,
    tx_types,
    case::geth_zero_transfer("geth", U256::from(0), vec!["legacy", "eip1559"]),
    case::proxy_zero_transfer("proxy", U256::from(0), vec!["legacy", "eip1559"]),
    case::geth_transfer("geth", U256::from(1000000000000000000u128), vec!["legacy", "eip1559"]),
    case::proxy_transfer("proxy", U256::from(1000000000000000000u128), vec!["legacy", "eip1559"])
)]
#[serial]
async fn transaction(
    provider_name: &str,
    airdrop_amount: U256,
    tx_types: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();

    let sender = get_random_wallet(); // Ethereum sender
    let _ = airdrop_to_address(sender.address(), U256::exp10(19), provider_name).await?;
    let receiver = get_receiver_wallet();

    println!("\n[ Info: ] - Checking state before transactions");
    check_state(vec![sender.address(), receiver.address()]).await;

    transfer_tx(
        tx_types, // "legacy"
        airdrop_amount,
        provider_name,
        sender,
        receiver.address(),
    )
    .await?;

    println!(
        "[ {:.2}s ] - Total duration\n",
        total_start.elapsed().as_secs_f64()
    );
    Ok(())
}

#[rstest(
    contract,
    provider_name,
    call_functions,
    slots,
    initial_values,
    values,
    // MultiStorage
    case::change_storage_uint_geth(
        "MultiStorage",
        "geth",
        Some(vec!["update"]),
        Some(vec![0, 1, 2, 5]), //  3, 4,
        Some(vec![0, 0, 0, 0]),
        Some(vec![12, 110, 1, 11]) //  989, 1434,
    ),
    case::change_storage_uint_proxy(
        "MultiStorage",
        "proxy",
        Some(vec!["update"]),
        Some(vec![0, 1, 2, 5]), //  3, 4,
        Some(vec![0, 0, 0, 0]),
        Some(vec![12, 110, 1, 11]) //  989, 1434,
    ),

    // CU
    case::change_storage_cu_push_geth(
        "CU",
        "geth",
        Some(vec!["push"]),
        Some(vec![0, 1]),
        Some(vec![0, 150]),
        Some(vec![100, 150]),
    ),
    case::change_storage_cu_push_proxy(
        "CU",
        "proxy",
        Some(vec!["push"]),
        Some(vec![0, 1]),
        Some(vec![0, 150]),
        Some(vec![100, 150]),
    ),
    case::change_storage_cu_update_geth(
        "CU",
        "geth",
        Some(vec!["update", "update_single"]),
        Some(vec![0, 1]),
        Some(vec![0, 150]),
        Some(vec![0, 150]),
    ),
    case::change_storage_cu_update_proxy(
        "CU",
        "proxy",
        Some(vec!["update", "update_single"]),
        Some(vec![0, 1]),
        Some(vec![0, 150]),
        Some(vec![0, 150]),
    ),
)]
#[serial]
async fn storage_check(
    contract: &str,
    provider_name: &str,
    call_functions: Option<Vec<&str>>,
    slots: Option<Vec<u64>>,
    initial_values: Option<Vec<u64>>,
    values: Option<Vec<u64>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();

    let (geth, proxy, sender, _receiver) = initial_setup().await;
    let provider = return_current_provider(provider_name);

    println!("\n[ Info: ] - Checking state before deployment");
    check_state(vec![sender.address()]).await;

    // Deploy contract
    let (contract, recipt) = deploy_contract(&contract, provider_name, &sender).await.unwrap();
    check_recipt(&recipt).await;
    check_state(vec![contract.address()]).await;

    // Change storage
    if call_functions != None {
        if initial_values != None {
            check_storage(
                provider.clone(),
                &proxy,
                &geth,
                contract.address(),
                slots.clone(),
                initial_values.clone(),
            )
            .await;
        }

        for call_function in call_functions.unwrap() {
            let start = Instant::now();
            // Call the contract method without an argument
            let tx_recipt = contract
                .method::<_, ()>(call_function, ())?
                .send()
                .await?
                .await?;
            println!(
                "[ {:.2}s ] - Calling function {}",
                start.elapsed().as_secs_f64(),
                call_function
            );

            println!("[ Info: ] - Checking recipt for {}", call_function);
            check_recipt(&tx_recipt.unwrap()).await;

            if values != None {
                check_storage(
                    provider.clone(),
                    &proxy,
                    &geth,
                    contract.address(),
                    slots.clone(),
                    values.clone(),
                )
                .await;
            }
            println!("[ Info: ] - Checking state after {}", call_function);
            check_state(vec![contract.address()]).await;
        }
    }

    println!(
        "[ {:.2}s ] - Total duration\n",
        total_start.elapsed().as_secs_f64()
    );
    Ok(())
}

#[rstest(
    contract,
    provider_name,
    call_functions,
    amount,

    case::withdraw_from_contract_geth("Caller", "geth", vec!["call1SOLWithdrawal"], U256::exp10(18)),
    case::withdraw_from_contract_proxy("Caller", "proxy", vec!["call1SOLWithdrawal"], U256::exp10(18)),
)]
#[serial]
async fn withdraw_test(
    contract: &str,
    provider_name: &str,
    call_functions: Vec<&str>,
    amount: U256,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();

    // Step 1: Init sender and fund it
    let test_account = test_account(); // receiver on Solana side
    let sender = get_random_wallet(); // Ethereum sender
    let _ = airdrop_to_address(sender.address(), U256::exp10(19), provider_name).await?; // fund sender
    let provider = return_current_provider(provider_name); // get eth provider
    let balance_sender_before = provider.get_balance(sender.address(), None).await?;
    let fee_recepient_balance_before = sum_fee_balances(provider_name).await;

    // Convert Solana test_account to hex for later withdrawal
    let address_bytes32 = test_account.to_bytes();
    let solana_receiver_address = address_bytes32
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    let solana_balance_before = solana_balance(test_account); // balance before on Solana

    // Step 2: State check before any actions
    println!("[ Info: ] - Checking state before deployment");
    check_state(vec![sender.address()]).await;

    // Step 3: Deploy test contract
    let (contract, recipt) = deploy_contract(&contract, provider_name, &sender)
        .await
        .unwrap();
    check_recipt(&recipt).await;
    check_state(vec![contract.address()]).await;

    // Check balances before topping up contract
    let custom_contract_balance_before = provider.get_balance(contract.address(), None).await?;
    let predeployed_address = H160::from_str(WITHDRAWAL_ADDRESS).unwrap(); // intermediate Ethereum receiver
    let balance_predeployed_before = provider.get_balance(predeployed_address, None).await?;

    // Step 4: Fund deployed contract
    transfer_tx(
        vec!["legacy"], // "legacy"
        amount,
        provider_name,
        sender.clone(),
        contract.address(),
    )
    .await?;
    let custom_contract_balance_airdrop = provider.get_balance(contract.address(), None).await?;

    assert_eq!(
        custom_contract_balance_airdrop - custom_contract_balance_before,
        amount
    );

    // Step 5: Call withdrawal function(s)
    for function in call_functions {
        let withdrawal_target = H256::from_slice(&hex::decode(&solana_receiver_address).unwrap());
        let tx_recipt = contract
            .method::<_, ()>(function, withdrawal_target)? // call withdrawal method
            .send()
            .await?
            .await?;

        println!("[ Info: ] - Checking recipt & state for {}", function);
        check_recipt(&tx_recipt.unwrap()).await;
        check_state(vec![contract.address()]).await;
    }

    // Step 6: Check balances after withdrawal
    let balance_sender_after = provider.get_balance(sender.address(), None).await?;
    let custom_contract_balance_after = provider.get_balance(contract.address(), None).await?;
    let balance_predeployed_after = provider.get_balance(predeployed_address, None).await?;
    let solana_balance_after = solana_balance(test_account);
    let fee_recepient_balance_after = sum_fee_balances(provider_name).await;
    let total_fee = fee_recepient_balance_after - fee_recepient_balance_before;

    // Step 7: Assert correctness of cross-chain transfer
    assert_eq!(
        balance_sender_after,
        balance_sender_before - total_fee - amount
    );
    assert_eq!(
        solana_balance_after,
        solana_balance_before + amount / U256::from(1_000_000_000)
    );
    assert_eq!(
        balance_predeployed_after,
        balance_predeployed_before + amount
    );
    assert_eq!(
        custom_contract_balance_after,
        custom_contract_balance_airdrop - amount
    );

    println!(
        "[ {:.2}s ] - Total duration\n",
        total_start.elapsed().as_secs_f64()
    );
    Ok(())
}

#[rstest(
    provider_name,
    amount,
    case::withdraw_raw_geth("geth", U256::exp10(18)),
    case::withdraw_raw_proxy("proxy", U256::exp10(18))
)]
#[serial]
async fn withdraw_raw_test(
    provider_name: &str,
    amount: U256,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();

    //Step 1: Initial setup
    let test_account = test_account(); // receiver on Solana side
    let sender = get_random_wallet(); // Ethereum sender
    let _ = airdrop_to_address(sender.address(), U256::exp10(19), provider_name).await?; // fund sender
    let provider = return_current_provider(provider_name);

    // Convert the Solana public key to bytes32 format (needed for the withdrawal call)
    let solana_address_bytes32 = test_account.to_bytes();
    let solana_balance_before = solana_balance(test_account);
    println!("\n[ Info: ] - Checking state before deployment");
    check_state(vec![sender.address()]).await;

    let predeployed_address = H160::from_str(WITHDRAWAL_ADDRESS).unwrap();
    let balance_predeployed_before = provider.get_balance(predeployed_address, None).await?;
    let balance_sender_before = provider.get_balance(sender.address(), None).await?;
    let fee_recepient_balance_before = sum_fee_balances(provider_name).await;

    // Wrap the provider and sender into a signing client
    let client = SignerMiddleware::new(provider.clone(), sender.clone());
    let client = Arc::new(client);

    //Step 2: Prepare the withdrawal call
    // Create the function selector for withdrawal(bytes32)
    let selector = ethers::utils::id("withdrawal(bytes32)")[..4].to_vec();
    // ABI-encode the Solana bytes32 address argument
    let encoded_args = ethers::abi::encode(&[ethers::abi::Token::FixedBytes(
        solana_address_bytes32.to_vec(),
    )]);
    // Concatenate selector and arguments to get the final calldata
    let calldata = Bytes::from([selector, encoded_args].concat());
    println!("[ Info: ] - Calling function withdraw");
    // Build the raw transaction to call the withdrawal function
    let tx = TransactionRequest::new()
        .to(predeployed_address)
        .data(calldata)
        .value(amount); // value sent with the transaction

    //Step 3: Execute transaction and wait for confirmation
    // Send the transaction and wait for 1 confirmation
    let pending_tx = client.send_transaction(tx, None).await?;
    let _receipt = pending_tx.confirmations(1).await?;

    //Step 4: Verify resulting balances
    let balance_predeployed_after = provider.get_balance(predeployed_address, None).await?;
    let solana_balance_after = solana_balance(test_account);
    let balance_sender_after = provider.get_balance(sender.address(), None).await?;
    let fee_recepient_balance_after = sum_fee_balances(provider_name).await;
    let total_fee = fee_recepient_balance_after - fee_recepient_balance_before;

    //Step 5: Assertions to verify correctness
    assert_eq!(
        balance_sender_after,
        balance_sender_before - amount - total_fee
    );
    assert_eq!(
        solana_balance_after,
        solana_balance_before + amount / U256::from(1_000_000_000)
    );
    assert_eq!(
        balance_predeployed_after,
        balance_predeployed_before + amount
    );

    println!(
        "[ {:.2}s ] - Total duration\n",
        total_start.elapsed().as_secs_f64()
    );
    Ok(())
}
