use std::fs;

use ed25519_dalek::PublicKey;
use rpc::{bump_contract_instance_tx, get_client, restore_contract_instance_tx};
use serde::{Serialize, Deserialize};

use clap::Parser;
use stellar_strkey::ed25519::PrivateKey;

#[derive(Debug, Clone)]
enum Action {
    Bump,
    Restore
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    secret: String,

    #[arg(short, long)]
    action: Action
}

mod rpc;

#[derive(Serialize, Deserialize)]
struct BumpSettings<'a> {
    contracts: Vec<String>,
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

    let secret = ed25519_dalek::SecretKey::from_bytes(&PrivateKey::from_string(&args.secret).unwrap().0).unwrap(); // TODO: error handling
    let public: PublicKey = (&secret).into();
    
    let keypair = ed25519_dalek::Keypair { secret, public };

    let rpc_client = get_client(parsed.rpc_url);
    
    let public_strkey = stellar_strkey::ed25519::PublicKey(public.to_bytes()).to_string();
    let account = rpc_client.get_account(&public_strkey).await.unwrap(); // TODO: error handling

    let mut contracts = Vec::new();

    for contract in parsed.contracts {
        let bytes = stellar_strkey::Contract::from_string(&contract).unwrap().0;
        contracts.push(bytes);
    }
    
    match args.action {
        Action::Bump => {
            let tx = bump_contract_instance_tx(public.to_bytes(), contracts, account.seq_num.0, parsed.min_ledgers_to_live);

            let response = rpc_client.prepare_and_send_transaction(&tx, &keypair, parsed.network, None).await;

            if let Ok(response) = response {
                let (result, meta, events) = response;
                println!("Bump was successful");

                // TODO: probably do more with response info in terms of logging.
            } else {
                println!("Error when submitting tx {:?}", response)
            }
        }

        Action::Restore => {
            let tx = restore_contract_instance_tx(public.to_bytes(), contracts, account.seq_num.0);
    
            let response = rpc_client.prepare_and_send_transaction(&tx, &keypair, parsed.network, None).await;

            if let Ok(response) = response {
                let (result, meta, events) = response;
                println!("Restore was successful");

                // TODO: probably do more with response info in terms of logging.
            } else {
                println!("Error when submitting tx {:?}", response)
            }
        }
    }

}
