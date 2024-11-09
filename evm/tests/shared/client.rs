use std::ops::Deref;

use rome_sdk::rome_evm_client::{
    RomeEVMClient,
    indexer::{
        solana_block_inmemory_storage::SolanaBlockInMemoryStorage,
        transaction_inmemory_storage::TransactionInMemoryStorage,
    },
};
use rome_sdk::rome_solana::indexers::clock::SolanaClockIndexer;
use rome_sdk::rome_solana::payer::SolanaKeyPayer;
use rome_sdk::rome_solana::tower::SolanaTower;
use rome_sdk::rome_solana::types::AsyncAtomicRpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use ethers_core::{k256::ecdsa::SigningKey, types::Address};
use ethers_core::types::transaction::eip2718::TypedTransaction;
use super::{config::Config, tx::{do_tx, do_rlp}};
use ethers_signers::{Signer as EthSigner, Wallet};
use rome_sdk::rome_evm_client::Payer;

use ethers_core::types::Bytes;
use crate::shared::{
    tx::{abi, calc_address, method_id}, CONTRACTS,
};
use std::str::FromStr;

/// [RomeEVMClient] and payer [Keypair]
pub struct Client {
    /// instance of [RomeEVMClient]
    pub client: RomeEVMClient<SolanaBlockInMemoryStorage, TransactionInMemoryStorage>,
    /// upgrade_authority keypair of the rome-evm contract
    pub upgrade_authority: Keypair,
    /// rollup owner
    pub rollup_owner: Keypair,
    /// rollup owner wallet
    pub rollup_owner_wallet: Wallet<SigningKey>,
}

impl Deref for Client {
    type Target = RomeEVMClient<SolanaBlockInMemoryStorage, TransactionInMemoryStorage>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl Client {
    /// Create a new instance of [Client] with the given [Config]
    pub async fn new(
        config: Config,
        rollup_owner_wallet: Wallet<SigningKey>,
    ) -> Client {

        let payers = Payer::from_config_list(&config.payers).await.unwrap();

        let program_id = Pubkey::from_str(&config.program_id).unwrap();

        let upgrade_authority = SolanaKeyPayer::read_from_file(&config.upgrade_authority_keypair)
            .await
            .expect("read upgrade-authority-keypair error")
            .into_keypair()
            .into();

        let rollup_owner = SolanaKeyPayer::read_from_file(&config.rollup_owner_keypair)
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
        let client = RomeEVMClient::new(
            config.chain_id,
            program_id,
            tower,
            config.solana.commitment,
            SolanaBlockInMemoryStorage::new(),
            TransactionInMemoryStorage::new(),
            payers,
        );

        Self {
            client: client,
            upgrade_authority,
            rollup_owner,
            rollup_owner_wallet,
        }
    }


    // #[allow(dead_code)]
    // #[inline]
    // pub fn get_rhea_sender(&self) -> &Keypair {
    //     &self.rhea_payer
    // }


    /// Sign and send transaction, check gas_transfer
    pub async fn send_tx(
        &self,
        tx: &TypedTransaction,
        wallet: &Wallet<SigningKey>,
        zero_gas: bool,
    ) {
        let rlp = do_rlp(tx, wallet);
        let from = wallet.address();

        let initial = self.get_balance(from).unwrap();
        assert!(initial >= tx.value().cloned().unwrap_or_default());

        self.send_transaction(rlp.into()).await.unwrap();

        let actual= self.get_balance(from).unwrap();
        let gas_transfer = if !zero_gas {
            let transfer = tx
                .gas()
                .cloned()
                .unwrap_or_default()
                .checked_mul(
                    tx.gas_price().unwrap_or_default()
                ).unwrap();
            assert!(transfer > 0.into());
            transfer
        } else {
            0.into()
        };

        let sum = gas_transfer
            .checked_add(
                tx.value().cloned().unwrap_or_default()
            ).unwrap();
        let expected = initial.saturating_sub(sum);
        let resource = self.client.tx_builder().lock_resource().await.unwrap();
        if let Some(fee_recipient) = resource.fee_recipient_address() {
            let gas_recv_bal = self.get_balance(fee_recipient).unwrap();

            assert_eq!(actual, expected);
            assert!(gas_recv_bal >= gas_transfer);
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
        zero_gas: bool,
    ) -> Address {
        let path = format!("{}{}.binary", CONTRACTS, contract);
        let mut bin = std::fs::read(&path).unwrap();
        if let Some(mut ctor) = ctor {
            bin.append(&mut ctor)
        }
        let tx = do_tx(self, None, bin, &wallet, 0, tx_type);
        let to = calc_address(self, &wallet.address());
        self.send_tx(&tx, wallet, zero_gas).await;

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
        zero_gas: bool,
    ) {
        let abi = abi(&format!("{}{}.abi", CONTRACTS, contract));
        let tx = do_tx(self, Some(*address), method_id(&abi, method), &wallet, 0, tx_type);

        self.send_tx(&tx, wallet, zero_gas).await;
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
        value: u64,
        zero_gas: bool,
    ) {
        let initial = self.get_balance(*to).unwrap().as_u64();

        let tx = do_tx(&self, Some(*to), vec![], &wallet, value, 0);
        Self::send_tx(&self, &tx, wallet, zero_gas).await;

        assert_eq!(self.get_balance(*to).unwrap().as_u64(),  initial + value );
    }

    /// Airdrop
    #[allow(dead_code)]
    pub async fn airdrop(&self, to: Address, value: u64, zero_gas: bool) {
        self.transfer(&self.rollup_owner_wallet, &to, value, zero_gas).await;
    }
}
