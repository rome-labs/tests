use std::ops::Deref;

use rome_sdk::rome_evm_client::{
    RomeEVMClient,
    indexer::inmemory::EthereumBlockStorage,
};
use rome_sdk::rome_solana::indexers::clock::SolanaClockIndexer;
use rome_sdk::rome_solana::payer::SolanaKeyPayer;
use rome_sdk::rome_solana::tower::SolanaTower;
use rome_sdk::rome_solana::types::AsyncAtomicRpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use ethers_core::{k256::ecdsa::SigningKey, types::Address};
use ethers_core::types::{
    transaction::eip2718::TypedTransaction, U256, H160,
};
use super::{config::Config, client_config, tx::{do_tx, do_rlp}};
use ethers_signers::{Signer as EthSigner, Wallet};
use rome_sdk::rome_evm_client::Payer;

use ethers_core::types::Bytes;
use crate::shared::{
    tx::{abi, calc_address, method_id}, CONTRACTS,
};
use std::str::FromStr;
use std::sync::Arc;
use crate::shared::utils::{run_on_testnet, run_on_devnet};
use crate::shared::fixture::{cfg_path};
type ClientType = RomeEVMClient;

/// [RomeEVMClient] and payer [Keypair]
pub struct Client {
    /// instance of [RomeEVMClient]
    pub sdk_client: ClientType,
    /// upgrade_authority keypair of the rome-evm contract
    pub upgrade_authority: Keypair,
    /// user's solana wallet
    pub user_solana_wallet: Keypair,
    /// user's rome wallet
    pub user_wallet: Wallet<SigningKey>,
}

impl Deref for Client {
    type Target = ClientType;

    fn deref(&self) -> &Self::Target {
        &self.sdk_client
    }
}

impl Client {
    /// Create a new instance of [Client] with the given [Config]
    pub async fn new(
        config: Config,
        user_wallet: Wallet<SigningKey>,
    ) -> Client {

        let payers = Payer::from_config_list(&config.payers).await.unwrap();

        let program_id = Pubkey::from_str(&config.program_id).unwrap();

        let upgrade_authority = if !run_on_testnet() && !run_on_devnet() {
            SolanaKeyPayer::read_from_file(config.upgrade_authority_keypair.as_ref().unwrap())
            .await
            .expect("read upgrade-authority-keypair error")
            .into_keypair()
        } else {
            Keypair::new()
        };

        let user_solana_wallet = if !run_on_testnet() && !run_on_devnet() {
            SolanaKeyPayer::read_from_file(config.user_keypair.as_ref().unwrap())
            .await
            .expect("read user_keypair error")
            .into_keypair()
        } else {
            Keypair::new()
        };

        let rpc_client: AsyncAtomicRpcClient = config.solana.clone().into_async_client().into();

        let solana_clock_indexer = SolanaClockIndexer::new(rpc_client.clone())
            .await
            .expect("create solana clock indexer error");

        let clock = solana_clock_indexer.get_current_clock();

        tokio::spawn(solana_clock_indexer.start());
        let tower = SolanaTower::new(rpc_client, clock);
        let ix_storage = Arc::new(EthereumBlockStorage);

        let sdk_client = RomeEVMClient::new(
            config.chain_id,
            program_id,
            tower,
            config.solana.commitment,
            ix_storage,
            payers,
            U256::exp10(9),
        );

        Self {
            sdk_client,
            upgrade_authority: upgrade_authority,
            user_solana_wallet: user_solana_wallet,
            user_wallet
        }
    }

    /// Sign and send transaction, check gas_transfer
    pub async fn send_tx(
        &self,
        tx: &TypedTransaction,
        wallet: &Wallet<SigningKey>,
    ) {
        let rlp = do_rlp(tx, wallet);
        let from = wallet.address();
        let initial = self.get_balance(from).unwrap();

        self.send_transaction(rlp.into()).await.unwrap();

        let actual= self.get_balance(from).unwrap();
        let transfer = initial.checked_sub(actual).unwrap();
        let resource = self.sdk_client.tx_builder().lock_resource().await.unwrap();

        if let Some(fee_recipient) = resource.fee_recipient_address() {
            let balance = self.get_balance(fee_recipient).unwrap();

            assert!(transfer > U256::zero());
            assert!(balance > U256::zero());
        }
    }

    /// Deploy contract
    #[allow(dead_code)]
    pub async fn deploy(
        &self,
        contract: &String,
        wallet: &Wallet<SigningKey>,
        ctor: Option<Vec<u8>>,
        tx_type: u8,
    ) -> Address {
        let path = format!("{}{}.binary", CONTRACTS, contract);
        let mut bin = std::fs::read(&path).unwrap();
        if let Some(mut ctor) = ctor {
            bin.append(&mut ctor)
        }
        let tx = do_tx(self, None, bin, &wallet, 0.into(), tx_type);
        let to = calc_address(self, &wallet.address());
        self.send_tx(&tx, wallet).await;

        to
    }

    // Call the contract method
    #[allow(dead_code)]
    pub async fn method_call (
        &self,
        contract: &String,
        address: &Address,
        method: &str,
        wallet: &Wallet<SigningKey>,
        value: U256,
        tx_type: u8,
    ) {
        let abi = abi(&format!("{}{}.abi", CONTRACTS, contract));
        let call_data = method_id(&abi, method);
        let tx = do_tx(self, Some(*address), call_data, &wallet, value, tx_type);
        self.send_tx(&tx, wallet).await;
    }

    #[allow(dead_code)]
    pub async fn raw_call (
        &self,
        address: &Address,
        method: &str,
        wallet: &Wallet<SigningKey>,
        value: U256,
        tx_type: u8,
        address_bytes32: [u8; 32],
    ) {
        let selector = ethers::utils::id(method)[..4].to_vec();
        let encoded_args = ethers::abi::encode(&[ethers::abi::Token::FixedBytes(address_bytes32.to_vec())]);
        let call_data = Bytes::from([selector, encoded_args].concat()).to_vec();
        let tx = do_tx(self, Some(*address), call_data, &wallet, value, tx_type);
        self.send_tx(&tx, wallet).await;
    }

    /// eth_Call
    #[allow(dead_code)]
    pub fn eth_call (
        &self,
        contract: &String,
        address: &Address,
        method: &str,
        wallet: &Wallet<SigningKey>,
    ) -> Bytes {
        let abi = abi(&format!("{}{}.abi", CONTRACTS, contract));
        let tx = do_tx(self, Some(*address), method_id(&abi, method), &wallet, 0.into(), 0);

        self.call(&tx.into()).unwrap()
    }

    /// Transfer funds
    #[allow(dead_code)]
    pub async fn transfer(
        &self,
        wallet: &Wallet<SigningKey>,
        to: &Address,
        value: U256,
    ) {
        let initial = self.get_balance(*to).unwrap();

        let tx = do_tx(&self, Some(*to), vec![], &wallet, value, 0);
        Self::send_tx(&self, &tx, wallet).await;

        assert_eq!(self.get_balance(*to).unwrap(),  initial + value );
    }

    /// Airdrop
    #[allow(dead_code)]
    pub async fn airdrop(&self, to: Address, value: U256) {
        if !run_on_testnet() && !run_on_devnet() {
            self.transfer(&self.user_wallet, &to, value).await;
        } else {
            println!("Run on testnet or devnet");
        }
    }

    #[allow(dead_code)]
    pub fn get_fee_addresses(zero_gas: bool) -> Vec<H160> {
        let config = client_config(cfg_path(zero_gas));
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
    pub async fn sum_fee_balances(&self, zero_gas: bool) -> U256 {
        let fee_addresses = Self::get_fee_addresses(zero_gas);
        let mut sum = U256::zero();
        for addr in &fee_addresses {
            sum += self.get_balance(*addr).unwrap();
        }
        sum
    }
}
