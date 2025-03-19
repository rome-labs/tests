mod pre_test;
mod post_test;
mod do_test;

use {
    std::{env, fs,},
    tokio::runtime::Builder,
    pre_test::*,
    post_test::*,
    do_test::*,
};

const OZ_CONTRACTS: &str = "/opt/oz/openzeppelin-contracts";
const RESULTS: &str = "/opt/results";
const ALLURE_RESULTS: &str = "/opt/allure-results";
const ALLURE_ENV: &str = "/opt/allure-results/environment.properties";

async fn oz (url: &str, tasks: usize, chain_id: u64, hh_acc_number: usize) {
    println!("Start OpenZeppelin tests, {} tasks, {} wallets per task, chain_id: {}, url: {}",
             tasks, hh_acc_number, chain_id, url);

    let files = load_tests();

    let genesis_private_key = env::var("GENESIS_PRIVATE_KEY").unwrap();

    let private_keys = create_private_keys(files.len(), hh_acc_number);
    airdrop(&private_keys, url, genesis_private_key).await;

    fs::create_dir(RESULTS).unwrap();

    let jhs = spawn( tasks, files, private_keys).await;

    for jh in jhs {
        jh.await.unwrap();
    }

    merge_time_logs();
    create_allure_env();
    fix_allure_results();
    report();
}

fn main() {

    let tasks = env::var("TASKS_NUMBER")
        .expect("TASKS_NUMBER env expected")
        .parse::<usize>()
        .unwrap();

    let url = env::var("PROXY_URL")
        .expect("PROXY_URL expected");

    let chain_id = env::var("NETWORK_ID")
        .expect("NETWORK_ID expected")
        .parse::<u64>()
        .unwrap();

    let hh_acc_number = env::var("HARDHAT_ACCOUNTS_NUMBER")
        .expect("HARDHAT_ACCOUNTS_NUMBER")
        .parse::<usize>()
        .unwrap();

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on( async move {
        oz(&url, tasks, chain_id, hh_acc_number).await;
    });
}
