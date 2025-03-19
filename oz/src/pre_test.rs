use {
    glob::glob,
    crate::OZ_CONTRACTS,
    tokio::process::Command,
    ethers_signers::{Signer, Wallet,},
    ethers::prelude::{k256::SecretKey, Address},
    tokio::time::{sleep, Duration},
};

pub fn load_tests() -> Vec<String> {

    let files = glob(&format!("{}/test/**/*.test.js", OZ_CONTRACTS))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let files = files.iter().map(|file| {
        format!("{}", file.as_os_str().to_str().unwrap())
    }).collect::<Vec<_>>();

    let priority = vec![
        "test/token/ERC721/ERC721.test.js",
        "test/token/ERC721/ERC721Enumerable.test.js",
        "test/token/ERC721/extensions/ERC721Wrapper.test.js",
    ];

    let mut first = vec![];
    let mut second = vec![];

    for file in files.into_iter() {
        if priority.iter().any(|a| file.ends_with(a)) {
            first.push(file.clone())
        } else  {
            second.push(file.clone())
        }
    }

    first.append(&mut second);

    first
}


fn address_from_private(private_key: &str) -> Address {
    let private_key_bytes = hex::decode(private_key).expect("Invalid hex string");
    let secret_key = SecretKey::from_slice(&private_key_bytes).expect("Invalid private key");
    Wallet::from(secret_key).address()
}

pub fn create_private_keys(tasks: usize, hh_acc_number: usize) -> Vec<Vec<String>> {
   let mut rng = rand_core::OsRng {};

   let mut f = |cnt| -> Vec<String> {
       let keys = (0..cnt)
           .map(|_| {
               let mut key = [0_u8; 32];
               rand_core::impls::fill_bytes_via_next(&mut rng, &mut key);
               key

           })
           .collect::<Vec<_>>();

       let hex_keys = keys
           .iter()
           .map(|key| {
               hex::encode(key)
           }).collect::<Vec<_>>();
       hex_keys
   };

    (0..tasks)
        .map(|_| f(hh_acc_number) )
        .collect::<Vec<_>>()
}


pub async fn airdrop(private_keys: &Vec<Vec<String>>, url: &str, genesis_pk: String) {

    for list in private_keys {
        for private_key in list {
            let address = address_from_private(private_key);

            let mut cmd = Command::new("sh");
            cmd.arg("-c");
            cmd.arg("web3 transfer 20000 to ".to_string() + &hex::encode(&address));
            cmd.env("WEB3_RPC_URL", url);
            cmd.env("WEB3_PRIVATE_KEY", &genesis_pk);

            let out = cmd.output().await.unwrap();
            let stdout = String::from_utf8(out.stdout).unwrap();
            let stderr = String::from_utf8(out.stderr).unwrap();

            println!("airdrop 20000 tokens to {}", address);
            println!("{}", &stdout);
            println!("{}", &stderr);

            sleep(Duration::from_millis(1000)).await;
        }
    }
}

