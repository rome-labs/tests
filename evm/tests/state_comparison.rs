mod shared;

use crate::shared::utils::{
    check_recipt, check_state, check_storage, deploy_contract, initial_setup, prepare_tx,
    return_current_provider,
};
use {
    ethers::{middleware::SignerMiddleware, providers::Middleware},
    ethers_core::types::U256,
    ethers_signers::Signer,
    rstest::*,
    serial_test::serial,
};
use std::time::Instant;

#[rstest(
    provider,
    airdrop_amount,
    tx_types,
    case::geth_zero_transfer("geth", 0, vec!["legacy", "eip1559"]),
    case::proxy_zero_transfer("proxy", 0, vec!["legacy", "eip1559"]),
    case::geth_transfer("geth", 10000000000000, vec!["legacy", "eip1559"]),
    case::proxy_transfer("proxy", 10000000000000, vec!["legacy", "eip1559"])
)]
#[serial]
async fn transaction(
    provider: &str,
    airdrop_amount: u64,
    tx_types: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();
    
    let (geth, proxy, sender, receiver) = initial_setup().await;
    let provider = return_current_provider(provider, geth.clone(), proxy.clone());

    println!("[ Info: ] - Checking state before transactions");
    check_state(&proxy, &geth, vec![sender.address(), receiver.address()]).await;
    
    let chain_id = provider.get_chainid().await.unwrap().as_u64().into();
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
            receiver.address(),
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
        println!("[ {:.2}s ] - Tx send: {}", start.elapsed().as_secs_f64(), receipt.transaction_hash);

        println!("[ Info: ] - Checking state after {}", tx_type);
        check_state(&proxy, &geth, vec![sender.address(), receiver.address()]).await;
    }

    println!("[ {:.2}s ] - Total duration\n", total_start.elapsed().as_secs_f64());
    Ok(())
}

#[rstest(
    contract,
    provider,
    call_functions,
    slots,
    initial_values,
    values,

    // HelloWorld
    case::deploy_hello_world_geth("HelloWorld", "geth", Some(vec!["hello_world"]), None, None, None),
    case::deploy_hello_world_proxy("HelloWorld", "proxy", Some(vec!["hello_world"]), None, None, None),

    // AB Deploy, #TODO: add "call_revert" and handle a panic
    case::deploy_ab_geth("A", "geth", Some(vec![ "update", "deploy_B", "call_update_slot", "get_B_address"]), None, None, None),
    case::deploy_ab_proxy("A", "proxy", Some(vec!["deploy_B", "update", "call_update_slot", "get_B_address"]), None, None, None),

    // AtomicIterative
    case::deploy_atomic_iterative_geth("AtomicIterative", "geth", Some(vec!["iterative_rw"]), None, None, None),
    case::deploy_atomic_iterative_proxy("AtomicIterative", "proxy", Some(vec!["iterative_rw"]), None, None, None),

    // GetStorageAt
    case::deploy_get_storage_at_geth("GetStorageAt", "geth", Some(vec!["get"]), None, None, None),
    case::deploy_get_storage_at_proxy("GetStorageAt", "proxy", Some(vec!["get"]), None, None, None),

    // IterativeOpGeth
    case::deploy_iterative_op_geth_geth("IterativeOpGeth", "geth", Some(vec!["iterative"]), None, None, None),
    case::deploy_iterative_op_geth_proxy("IterativeOpGeth", "proxy", Some(vec!["iterative"]), None, None, None),

    // NestedCall
    // case::deploy_NestedCall_geth("NestedCall", "geth", None, None, None, None),
    // case::deploy_NestedCall_proxy("NestedCall", "proxy", None, None, None, None),

    // DestructCaller
    case::deploy_destruct_caller_geth("DestructCaller", "geth", None, None, None, None),
    case::deploy_destruct_caller_proxy("DestructCaller", "proxy", None, None, None, None),

    // TouchStorage
    case::deploy_touch_storage_geth("TouchStorage", "geth", Some(vec!["get_local"]), None, None, None),
    case::deploy_touch_storage_proxy("TouchStorage", "proxy", Some(vec!["get_local"]), None, None, None),

    // TestTransientStorage
    case::deploy_test_transient_storage_geth("TestTransientStorage", "geth", None, None, None, None),
    case::deploy_test_transient_storage_proxy("TestTransientStorage", "proxy", None, None, None, None),

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
async fn check_contract(
    contract: &str,
    provider: &str,
    call_functions: Option<Vec<&str>>,
    slots: Option<Vec<u64>>,
    initial_values: Option<Vec<u64>>,
    values: Option<Vec<u64>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();

    let (geth, proxy, sender, _receiver) = initial_setup().await;
    let provider = return_current_provider(provider, geth.clone(), proxy.clone());
    
    println!("[ Info: ] - Checking state before deployment");
    check_state(&proxy, &geth, vec![sender.address()]).await;

    // Deploy contract
    let (contract, recipt) = deploy_contract(&contract, provider.clone(), sender.clone())
        .await
        .unwrap();
    check_recipt(&proxy, &geth, &recipt).await;
    check_state(&proxy, &geth, vec![contract.address()]).await;

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
            println!("[ {:.2}s ] - Calling function {}", start.elapsed().as_secs_f64(), call_function);
            
            println!("[ Info: ] - Checking recipt for {}", call_function);
            check_recipt(&proxy, &geth, &tx_recipt.unwrap()).await;

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
           check_state(&proxy, &geth, vec![contract.address()]).await;
        }
    }

    println!("[ {:.2}s ] - Total duration\n", total_start.elapsed().as_secs_f64());
    Ok(())
}
