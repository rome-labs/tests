mod shared;

use crate::shared::utils::{
    check_recipt, check_state, deploy_contract, airdrop_to_address,
};
use shared::{
    utils::get_random_wallet,
};
use std::time::Instant;
use {
    ethers_core::types::{U256},
    ethers_signers::Signer,
    rstest::*,
    serial_test::serial,

};

#[rstest(
  contract,
  provider_name,
  call_functions,

  // HelloWorld
  case::deploy_hello_world_geth("HelloWorld", "geth", Some(vec!["hello_world"])),
  case::deploy_hello_world_proxy("HelloWorld", "proxy", Some(vec!["hello_world"])),

  // AB Deploy, #TODO: add "call_revert" and handle a panic
  case::deploy_ab_geth("A", "geth", Some(vec![ "update", "deploy_B", "call_update_slot", "get_B_address"])),
  case::deploy_ab_proxy("A", "proxy", Some(vec!["deploy_B", "update", "call_update_slot", "get_B_address"])),

  // AtomicIterative
  case::deploy_atomic_iterative_geth("AtomicIterative", "geth", Some(vec!["iterative_rw"])),
  case::deploy_atomic_iterative_proxy("AtomicIterative", "proxy", Some(vec!["iterative_rw"])),

  // GetStorageAt
  // TODO: figure out the problem
  // case::deploy_get_storage_at_geth("GetStorageAt", "geth", Some(vec!["get"])),
  // case::deploy_get_storage_at_proxy("GetStorageAt", "proxy", Some(vec!["get"])),

  // IterativeOpGeth
  case::deploy_iterative_op_geth_geth("IterativeOpGeth", "geth", Some(vec!["iterative"])),
  case::deploy_iterative_op_geth_proxy("IterativeOpGeth", "proxy", Some(vec!["iterative"])),

  // NestedCall
  // case::deploy_NestedCall_geth("NestedCall", "geth", None),
  // case::deploy_NestedCall_proxy("NestedCall", "proxy", None),

  // DestructCaller
  case::deploy_destruct_caller_geth("DestructCaller", "geth", None),
  case::deploy_destruct_caller_proxy("DestructCaller", "proxy", None),

  // TouchStorage
  case::deploy_touch_storage_geth("TouchStorage", "geth", Some(vec!["get_local"])),
  case::deploy_touch_storage_proxy("TouchStorage", "proxy", Some(vec!["get_local"])),

  // TestTransientStorage
  case::deploy_test_transient_storage_geth("TestTransientStorage", "geth", None),
  case::deploy_test_transient_storage_proxy("TestTransientStorage", "proxy", None),
)]
#[serial]
async fn check_contract(
  contract: &str,
  provider_name: &str,
  call_functions: Option<Vec<&str>>,
) -> Result<(), Box<dyn std::error::Error>> {
  let total_start = Instant::now();

  let sender = get_random_wallet();
  let _ = airdrop_to_address(sender.address(), U256::exp10(18), provider_name).await?; // fund sender

  println!("\n[ Info: ] - Checking state before deployment");
  check_state(vec![sender.address()]).await;

  // Deploy contract
  let (contract, recipt) = deploy_contract(&contract, provider_name, &sender).await.unwrap();
  check_recipt(&recipt).await;
  check_state(vec![contract.address()]).await;

  // Change storage
  if call_functions != None {
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
