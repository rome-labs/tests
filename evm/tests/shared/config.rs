use std::borrow::Cow;
use rome_sdk::rome_solana::config::SolanaConfig;
use std::{
    fs::File, io, path::Path,
    path::PathBuf
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Config {
    pub solana: SolanaConfig,
    pub chain_id: u64,
    pub program_keypair: PathBuf,
    pub payer_keypair: PathBuf,
    pub upgrade_authority_keypair: PathBuf,
    pub rollup_owner_keypair: PathBuf,
    pub rhea_sender_keypair: PathBuf,
    pub start_slot: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Keys<'a> {
    pub sender_public_key: Cow<'a, str>,
    pub recipient_public_key: Cow<'a, str>,
    pub sender_private_key: Cow<'a, str>,
    pub recipient_private_key: Cow<'a, str>,
}

pub fn load_config<T, P>(config_file: P) -> Result<T, io::Error>
where
    T: serde::de::DeserializeOwned,
    P: AsRef<Path> +  std::fmt::Debug + Copy,
{
    let file = File::open(config_file).expect(&format!("config file not found: {:?}", config_file));
    let config = serde_yaml::from_reader(file)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;
    Ok(config)
}
