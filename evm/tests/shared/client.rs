use std::ops::Deref;

use rome_sdk::rome_evm_client::RomeEVMClient;
use rome_sdk::rome_solana::indexers::clock::SolanaClockIndexer;
use rome_sdk::rome_solana::payer::SolanaKeyPayer;
use rome_sdk::rome_solana::tower::SolanaTower;
use rome_sdk::rome_solana::types::AsyncAtomicRpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use ethers_core::{k256::ecdsa::SigningKey, types::Address};
use ethers_core::types::transaction::eip2718::TypedTransaction;
use super::{config::Config, tx::do_tx, };
use ethers_signers::{Signer as EthSigner, Wallet};

use ethers_core::types::Bytes;
use crate::shared::{
    tx::{abi, calc_address, method_id}, CONTRACTS,
};
use std::sync::Mutex;

/// [RomeEVMClient] and payer [Keypair]
pub struct Client {
    /// instance of [RomeEVMClient]
    pub client: RomeEVMClient,
    /// Payer
    payer: Keypair,
    /// upgrade_authority keypair of the rome-evm contract
    pub upgrade_authority: Keypair,
    /// rollup owner
    pub rollup_owner: Keypair,
    /// rollup owner wallet
    pub rollup_owner_wallet: Wallet<SigningKey>,
    /// Rhea-keypair
    pub rhea_payer: Keypair,
    /// gas recipient address
    pub gas_recipient: Address,
    /// wallets
    pub lock: Mutex<u8>
}

impl Deref for Client {
    type Target = RomeEVMClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl Client {
    /// Create a new instance of [Client] with the given [Config]
    pub async fn new(
        config: Config,
        rollup_owner_wallet: Wallet<SigningKey>,
        gas_recipient: Address,
    ) -> Client {

        let program_id = SolanaKeyPayer::read_from_file(&config.program_keypair)
            .await
            .expect("read program-keypair error")
            .into_keypair()
            .pubkey();

        let payer = SolanaKeyPayer::read_from_file(&config.payer_keypair)
            .await
            .expect("read payer-keypair error")
            .into_keypair()
            .into();

        let rhea_payer = SolanaKeyPayer::read_from_file(&config.rhea_sender_keypair)
            .await
            .expect("read rhea-sender error")
            .into_keypair()
            .into();

        let  upgrade_authority = SolanaKeyPayer::read_from_file(&config.upgrade_authority_keypair)
            .await
            .expect("read upgrade-authority-keypair error")
            .into_keypair()
            .into();

        let  rollup_owner = SolanaKeyPayer::read_from_file(&config.rollup_owner_keypair)
            .await
            .expect("read rollup-owner-keypair error")
            .into_keypair()
            .into();

        let client: AsyncAtomicRpcClient = config.solana.clone().into_async_client().into();

        let solana_clock_indexer = SolanaClockIndexer::new(client.clone())
            .await
            .expect("create solana clock indexer error");

        let clock = solana_clock_indexer.get_current_clock();

        tokio::spawn(solana_clock_indexer.start());
        let tower = SolanaTower::new(client, clock);
        let client = RomeEVMClient::new(config.chain_id, program_id, tower, config.solana.commitment);

        Self {
            client: client,
            payer,
            upgrade_authority,
            rollup_owner,
            rollup_owner_wallet,
            rhea_payer,
            gas_recipient,
            lock: Mutex::new(0),
        }
    }

    /// Get the payer [Keypair]
    #[allow(dead_code)]
    #[inline]
    pub fn get_payer(&self) -> &Keypair {
        &self.payer
    }

    #[allow(dead_code)]
    #[inline]
    pub fn get_rhea_sender(&self) -> &Keypair {
        &self.rhea_payer
    }

    // Get the payer [Pubkey]
    #[allow(dead_code)]
    #[inline]
    pub fn get_payer_pubkey(&self) -> Pubkey {
        self.get_payer().pubkey()
    }

    /// Sign and send transaction, check gas_transfer
    pub async fn send_tx(&self, tx: &TypedTransaction, wallet: &Wallet<SigningKey>) {
        let sig = wallet.sign_transaction_sync(&tx).unwrap();
        let rlp = tx.rlp_signed(&sig);

        let from = wallet.address();

        let initial = self.get_balance(from).unwrap();
        assert!(initial >= tx.value().cloned().unwrap_or_default());

        self.send_transaction(rlp.into(), self.get_payer()).await.unwrap();

        let actual= self.get_balance(from).unwrap();
        let gas_transfer = tx
            .gas()
            .cloned()
            .unwrap_or_default()
            .checked_mul(
                tx.gas_price().unwrap_or_default()
            ).unwrap();

        let sum = gas_transfer
            .checked_add(
                tx.value().cloned().unwrap_or_default()
            ).unwrap();
        let expected = initial.saturating_sub(sum);
        let gas_recv_bal = self.get_balance(self.gas_recipient).unwrap();

        assert!(gas_transfer > 0.into());
        assert_eq!(actual, expected);
        assert!(gas_recv_bal >= gas_transfer);
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
        let tx = do_tx(self, None, bin, &wallet, 0, tx_type);
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
        tx_type: u8,
    ) {
        let abi = abi(&format!("{}{}.abi", CONTRACTS, contract));
        let tx = do_tx(self, Some(*address), method_id(&abi, method), &wallet, 0, tx_type);

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
        let tx = do_tx(self, Some(*address), method_id(&abi, method), &wallet, 0, 0);

        self.call(&tx.into()).unwrap()
    }

    /// Transfer funds
    #[allow(dead_code)]
    pub async fn transfer(
        &self,
        wallet: &Wallet<SigningKey>,
        to: &Address,
        value: u64
    ) {
        let initial = self.get_balance(*to).unwrap().as_u64();

        let tx = do_tx(&self, Some(*to), vec![], &wallet, value, 0);
        Self::send_tx(&self, &tx, wallet).await;

        assert_eq!(self.get_balance(*to).unwrap().as_u64(),  initial + value );
    }

    /// Airdrop
    #[allow(dead_code)]
    pub async fn airdrop(&self, to: Address, value: u64) {
        match self.lock.lock() {
            Ok(_) => {
                self.transfer(&self.rollup_owner_wallet, &to, value).await;
            },
            Err(e) => {
                panic!("mutex lock error {}", e);
            }
        }
    }
}
