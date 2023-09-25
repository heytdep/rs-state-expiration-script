use std::fs;
use soroban_env_host::xdr::WriteXdr;
use rpc::{get_client, restore_contract_instance_tx, bump_tx};
use serde::{Serialize, Deserialize};

use clap::Parser;
use stellar_strkey::ed25519::PrivateKey;

#[derive(Debug, Clone)]
enum Action {
    Bump,
    Restore
}

#[derive(Debug, Clone)]
pub enum Target {
    Instance,
    Code,
}


impl From<String> for Action {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Bump" => Action::Bump,
            "Restore" => Action::Restore,
            _ => panic!("Invalid action string"),
        }
    }
}

impl From<String> for Target {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Instance" => Target::Instance,
            "Code" => Target::Code,
            _ => panic!("Invalid target string"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    secret: String,

    #[arg(short, long)]
    action: Action,

    #[arg(short, long)]
    target: Target,
    
}

mod rpc;

#[derive(Serialize, Deserialize)]
struct BumpSettings<'a> {
    contracts: Option<Vec<String>>,
    hashes: Option<Vec<String>>,
    min_ledgers_to_live: u32,
    rpc_url: &'a str,
    network: &'a str,

    // ...
    // add more settings as we see fit
    // ... 
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let file = fs::read_to_string("./bump-settings.json").expect("failed to read bump-settings.json");
    let parsed: BumpSettings = serde_json::from_str(&file).unwrap();
    
    let private = PrivateKey::from_string(&args.secret).unwrap();    
    let keypair = ed25519_dalek::SigningKey::from_bytes(&private.0);

    let public = {
        let verifiying = keypair.verifying_key();
        verifiying.as_bytes().clone()
    };

    let rpc_client = get_client(parsed.rpc_url);
    
    let public_strkey = stellar_strkey::ed25519::PublicKey(public).to_string();
    let account = rpc_client.get_account(&public_strkey).await.unwrap(); // TODO: error handling
    
    match args.action {
        Action::Bump => {
            let tx = bump_tx(args.target, public, parsed.contracts, parsed.hashes, account.seq_num.0, parsed.min_ledgers_to_live).await;

	    println!("tx: {}", tx.to_xdr_base64().unwrap());

            let response = rpc_client.prepare_and_send_transaction(&tx, &keypair, &[], parsed.network, None, None).await;

            if let Ok(response) = response {
                let (result, meta, events) = response;
                println!("Bump was successful");
                print!("{}", serde_json::to_string_pretty(&result).unwrap())

                // TODO: probably do more with response info in terms of logging.
            } else {
                println!("Error when submitting tx {:?}", response)
            }
        }

        Action::Restore => {
            let tx = restore_contract_instance_tx(args.target, public, parsed.contracts, parsed.hashes, account.seq_num.0).await;
    
            let response = rpc_client.prepare_and_send_transaction(&tx, &keypair, &[], parsed.network, None, None).await;

            if let Ok(response) = response {
                let (result, meta, events) = response;
                println!("Restore was successful");
                print!("{}", serde_json::to_string_pretty(&result).unwrap())

                // TODO: probably do more with response info in terms of logging.
            } else {
                println!("Error when submitting tx {:?}", response)
            }
        }
    }

}
