mod shared;

use crate::shared::utils::{
    create_spl_account, deploy_contract, get_account_state, get_random_wallet, get_receiver_wallet,
    get_solana_key, get_spl, mint_to, transfer, airdrop_to_address, sum_fee_balances,
};

use crate::shared::{ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, SPL_TOKEN_ID, MINT_ADDRESS};
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use std::time::Instant;
use {ethers_signers::Signer, rstest::*, serial_test::serial};
use ethers_core::types::U256;

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

#[rstest(provider_str, case::wsystem_program("proxy"))]
#[serial]
async fn verify_spl_account(provider_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();

    // Solana keys
    let sender = get_random_wallet(); // Ethereum sender
    let _ = airdrop_to_address(sender.address(), U256::exp10(19), provider_str).await?; // fund sender
    let receiver = get_receiver_wallet();
    let solana_sender_key = get_solana_key(sender.address());
    let solana_receiver_key = get_solana_key(receiver.address());
    println!("\n[ Info: ] - Solana Sender Address: {}", solana_sender_key);
    println!(
        "[ Info: ] - Solana Receiver Address: {}",
        solana_receiver_key
    );

    // Mint address
    let token_mint_key: Pubkey = Pubkey::from_str(MINT_ADDRESS).unwrap();

    // Calculate SPL account
    let spl_sender = get_spl(
        &solana_sender_key,                // seed
        &token_mint_key,                   // seed
        &ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, // aspl_token program_id
        &SPL_TOKEN_ID,                     // seed
    );
    let spl_receiver = get_spl(
        &solana_receiver_key,
        &token_mint_key,
        &ASSOCIATED_TOKEN_ACCOUNT_PROGRAM,
        &SPL_TOKEN_ID,
    );
    println!("[ Info: ] - Spl Sender Address: {}", spl_sender.0);
    println!("[ Info: ] - Spl Receiver Address: {}", spl_receiver.0);

    // Deploy SPL contracts
    let contract_name = "WAssociatedSplToken"; // to call "create_associated_token_account"
    let (contract_wassociated, _recipt2) =
        deploy_contract(&contract_name, provider_str, &sender).await.unwrap();

    let _ = create_spl_account(
        provider_str,
        solana_sender_key,
        token_mint_key,
        contract_wassociated.address(),
        sender.clone(),
    )
    .await;
    let _ = create_spl_account(
        provider_str,
        solana_receiver_key,
        token_mint_key,
        contract_wassociated.address(),
        sender.clone(),
    )
    .await;

    let contract_name_wspl = "WSplToken";
    let (contract_spl, _recipt) = deploy_contract(&contract_name_wspl, provider_str, &sender)
        .await
        .unwrap();

    // Call `account_state`
    let result_sender = get_account_state(
        contract_name_wspl,
        contract_spl.address(),
        spl_sender.0,
        provider_str,
        sender.clone(),
    )
    .await?;

    let result_receiver = get_account_state(
        contract_name_wspl,
        contract_spl.address(),
        spl_sender.0,
        provider_str,
        receiver.clone(),
    )
    .await?;
    println!("[ Info: ] - Owner: {:?}", result_sender.owner);

    assert_eq!(result_sender.owner, solana_sender_key.to_string());
    assert_eq!(result_receiver.owner, solana_sender_key.to_string());

    println!(
        "[ {:.2}s ] - Total duration\n",
        total_start.elapsed().as_secs_f64()
    );
    Ok(())
}

#[rstest(provider_str, case::wsystem_program("proxy"))]
#[serial]
async fn verify_spl_transfer(provider_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();
    let sender = get_random_wallet(); // Ethereum sender
    let _ = airdrop_to_address(sender.address(), U256::exp10(19), provider_str).await?; // fund sender
    let _fee_recepient_balance_before = sum_fee_balances(provider_str).await;
    // Deploy SplHolderT contract
    let contract_spl_holder_t_name = "SplHolderT";
    let (contract_spl_holder_t, _recipt) =
        deploy_contract(&contract_spl_holder_t_name, provider_str, &sender)
            .await
            .unwrap();
    println!(
        "\n[ Info: ] - {} Address: {}",
        contract_spl_holder_t_name,
        contract_spl_holder_t.address()
    );

    // Solana keys
    let receiver = get_random_wallet();
    let solana_receiver_key = get_solana_key(receiver.address());
    let solana_contract_key = get_solana_key(contract_spl_holder_t.address());
    println!("[ Info: ] - Solana Sender Address: {}", solana_contract_key);
    println!(
        "[ Info: ] - Solana Receiver Address: {}",
        solana_contract_key
    );

    // Mint address
    let token_mint_key: Pubkey = Pubkey::from_str(MINT_ADDRESS).unwrap();

    // Calculate SPL account
    let spl_sender = get_spl(
        &solana_contract_key,
        &token_mint_key,
        &ASSOCIATED_TOKEN_ACCOUNT_PROGRAM,
        &SPL_TOKEN_ID,
    );

    let spl_receiver = get_spl(
        &solana_receiver_key,
        &token_mint_key,
        &ASSOCIATED_TOKEN_ACCOUNT_PROGRAM,
        &SPL_TOKEN_ID,
    );

    println!("[ Info: ] - Spl Sender Address: {}", spl_sender.0);
    println!("[ Info: ] - Spl Receiver Address: {}", spl_receiver.0);

    // Deploy SPL contracts
    let contract_name = "WAssociatedSplToken"; // to call "create_associated_token_account"
    let (contract_wassociated, _recipt2) =
        deploy_contract(&contract_name, provider_str, &sender).await.unwrap();

    let _ = create_spl_account(
        provider_str,
        solana_contract_key,
        token_mint_key,
        contract_wassociated.address(),
        sender.clone(),
    )
    .await?;
    let _ = create_spl_account(
        provider_str,
        solana_receiver_key,
        token_mint_key,
        contract_wassociated.address(),
        sender.clone(),
    )
    .await?;
        
    let contract_name_wspl = "WSplToken";
    let (contract_spl, _recipt) = deploy_contract(&contract_name_wspl, provider_str, &sender)
        .await
        .unwrap();

    let amount = "123";
    mint_to(MINT_ADDRESS, amount, spl_sender.0);

    // Call `account_state`
    let result_sender = get_account_state(
        contract_name_wspl,
        contract_spl.address(),
        spl_sender.0,
        provider_str,
        sender.clone(),
    )
    .await?;

    let result_receiver = get_account_state(
        contract_name_wspl,
        contract_spl.address(),
        spl_receiver.0,
        provider_str,
        receiver.clone(),
    )
    .await?;

    println!("[ Info: ] - Amount receiver: {:?}", result_receiver.amount);
    println!("[ Info: ] - Amount sender: {:?}", result_sender.amount);

    let decimals = 9; // Number of decimals for the token (e.g., 9 for Solana tokens)
    let scaled_amount = amount.parse::<u64>().unwrap() * 10u64.pow(decimals);
    let _fee_balance_after = sum_fee_balances(provider_str).await;

    assert_eq!(result_sender.amount, scaled_amount);

    let new_amount = 100u64;
    let _ = transfer(
        contract_spl_holder_t.address(),
        spl_sender.0,
        spl_receiver.0,
        new_amount,
        sender.clone(),
    )
    .await;

    // Re-Call `account_state`
    let result_sender = get_account_state(
        contract_name_wspl,
        contract_spl.address(),
        spl_sender.0,
        provider_str,
        sender.clone(),
    )
    .await?;

    let result_receiver = get_account_state(
        contract_name_wspl,
        contract_spl.address(),
        spl_receiver.0,
        provider_str,
        receiver.clone(),
    )
    .await?;

    println!(
        "[ Info: ] - amount result_sender: {:?}",
        result_sender.amount
    );
    println!(
        "[ Info: ] - amount result_receiver: {:?}",
        result_receiver.amount
    );
    // assert_eq!(result_sender.amount, scaled_amount - new_amount);
    assert_eq!(result_receiver.amount, new_amount);

    println!(
        "[ {:.2}s ] - Total duration\n",
        total_start.elapsed().as_secs_f64()
    );

    Ok(())
}
