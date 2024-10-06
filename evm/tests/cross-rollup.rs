mod shared;

use {
    rstest::rstest,
    shared::{
        config::{load_config, CrossRollupConfig},
        cross_rollup_fn::{
            compose_and_send_tx, get_balance_on_contract, transact_token_on_rollup,
            CROSS_ROLLUP_CONFIG_PATH,
        },
    },
    tracing::log::info,
};

#[rstest]
async fn test_cross_rollup_tx() -> Result<(), Box<dyn std::error::Error>> {
    let cross_rollup_config: CrossRollupConfig =
        load_config(CROSS_ROLLUP_CONFIG_PATH).expect("Config missing on path.");

    let (proxy1_endpoint, proxy2_endpoint) = match cross_rollup_config.proxy_endpoints.as_slice() {
        [first, second, ..] => (first, second),
        _ => panic!("Expected exactly two proxy endpoints in the configuration"),
    };
    let (chain_id_1, chain_id_2) = match cross_rollup_config.chain_ids.as_slice() {
        [first, second, ..] => (*first, *second),
        _ => panic!("Expected exactly two chain ids in the configuration"),
    };

    let sender_public_key = &cross_rollup_config.keys.sender_public_key;
    let sender_private_key = &cross_rollup_config.keys.sender_private_key;
    let recipient_public_key = &cross_rollup_config.keys.recipient_public_key;
    let token_a_contract_address_rollup_1 = &cross_rollup_config.token_a_contract_addresses[0];
    let token_b_contract_address_rollup_1 = &cross_rollup_config.token_b_contract_addresses[0];
    let token_a_contract_address_rollup_2 = &cross_rollup_config.token_a_contract_addresses[1];
    let token_b_contract_address_rollup_2 = &cross_rollup_config.token_b_contract_addresses[1];

    // Proxy 1
    let (tx_1, (sender_balance, _)) = transact_token_on_rollup(
        sender_private_key,
        recipient_public_key,
        proxy1_endpoint,
        token_a_contract_address_rollup_1,
        chain_id_1,
    )
    .await?;

    let token_a_balance_rollup_1_before_tx = sender_balance;
    let token_b_balance_rollup_1_before_tx = get_balance_on_contract(
        sender_private_key,
        token_b_contract_address_rollup_1,
        proxy1_endpoint,
        sender_public_key,
        chain_id_1,
    )
    .await?;

    info!(
        "Token A balance on rollup 1 before tx: {}",
        token_a_balance_rollup_1_before_tx
    );
    info!(
        "Token B balance on rollup 1 before tx: {}",
        token_b_balance_rollup_1_before_tx
    );

    // Proxy 2
    let (tx_2, (sender_balance_1, _)) = transact_token_on_rollup(
        sender_private_key,
        recipient_public_key,
        proxy2_endpoint,
        token_a_contract_address_rollup_2,
        chain_id_2,
    )
    .await?;

    let token_a_balance_rollup_2_before_tx = sender_balance_1;
    let token_b_balance_rollup_2_before_tx = get_balance_on_contract(
        sender_private_key,
        token_b_contract_address_rollup_2,
        proxy1_endpoint,
        sender_public_key,
        chain_id_2,
    )
    .await?;

    info!(
        "Token A balance on rollup 2 before tx: {}",
        token_a_balance_rollup_2_before_tx
    );
    info!(
        "Token B balance on rollup 2 before tx: {}",
        token_b_balance_rollup_2_before_tx
    );

    let _ = compose_and_send_tx(vec![tx_1, tx_2]).await?;

    // Query token A balance of account 1 on rollup 1
    let token_a_balance_rollup_1_after_tx = get_balance_on_contract(
        sender_private_key,
        token_a_contract_address_rollup_1,
        proxy1_endpoint,
        sender_public_key,
        chain_id_1,
    )
    .await?;

    // Query token B balance of account 1 on rollup 1
    let token_b_balance_rollup_1_after_tx = get_balance_on_contract(
        sender_private_key,
        token_b_contract_address_rollup_1,
        proxy1_endpoint,
        sender_public_key,
        chain_id_1,
    )
    .await?;

    // Query token A balance of account 1 on rollup 2
    let token_a_balance_rollup_2_after_tx = get_balance_on_contract(
        sender_private_key,
        token_a_contract_address_rollup_2,
        proxy2_endpoint,
        sender_public_key,
        chain_id_2,
    )
    .await?;

    // Query token B balance of account 1 on rollup 2
    let token_b_balance_rollup_2_after_tx = get_balance_on_contract(
        sender_private_key,
        token_b_contract_address_rollup_2,
        proxy2_endpoint,
        sender_public_key,
        chain_id_2,
    )
    .await?;

    info!(
        "Token A balance on rollup 1 after tx: {}",
        token_a_balance_rollup_1_after_tx
    );
    info!(
        "Token B balance on rollup 1 after tx: {}",
        token_b_balance_rollup_1_after_tx
    );
    info!(
        "Token A balance on rollup 2 after tx: {}",
        token_a_balance_rollup_2_after_tx
    );
    info!(
        "Token B balance on rollup 2 after tx: {}",
        token_b_balance_rollup_2_after_tx
    );

    Ok(())
}
