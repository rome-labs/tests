use {
    solana_sdk::commitment_config::CommitmentLevel,
    std::{fs::File, io, path::Path},
    serde::{Serialize, Deserialize},
};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub chain_id: u64,
    pub solana_url: String,
    pub commitment_level: CommitmentLevel,
    pub program_id_keypairs: Vec<String>,
    pub payer_keypair: String,
    pub log: String,
    pub host: String,
    pub number_holders: u64,
    pub start_slot: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrossRollupConfig {
    pub proxy_endpoints: Vec<String>,
    pub chain_ids: Vec<u64>,
    pub rome_config_path: String,
    pub token_a_contract_addresses: Vec<String>,
    pub token_b_contract_addresses: Vec<String>,
    pub keys: Keys,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Keys {
    pub sender_public_key: String,
    pub recipient_public_key: String,
    pub sender_private_key: String,
    pub recipient_private_key: String,
}

pub fn load_config<T, P>(config_file: P) -> Result<T, io::Error>
where
    T: serde::de::DeserializeOwned,
    P: AsRef<Path>,
{
    let file = File::open(config_file).expect("config file not found");
    let config = serde_yaml::from_reader(file)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;
    Ok(config)
}
