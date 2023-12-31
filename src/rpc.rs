use hex::FromHex;
use soroban_cli::rpc::Client;
use soroban_env_host::xdr::{Transaction, MuxedAccount, Uint256, SequenceNumber, Preconditions, Memo, Operation, OperationBody, BumpFootprintExpirationOp, ExtensionPoint, TransactionExt, SorobanTransactionData, SorobanResources, LedgerFootprint, LedgerKey, LedgerKeyContractData, ScAddress, Hash, ContractDataDurability, ScVal, RestoreFootprintOp, LedgerKeyContractCode, WriteXdr, ReadXdr, LedgerEntryData, ContractExecutable};
use serde_json::json;

use crate::Target;

const DEFAULT_FEE: u32 = 100;

fn build_bump_tx(public: [u8; 32], sequence: i64, parsed_keys: Vec<LedgerKey>, min_ledgers_to_live: u32) -> Transaction {
    Transaction {
        source_account: MuxedAccount::Ed25519(Uint256(public)),
        fee: DEFAULT_FEE,
        seq_num: SequenceNumber(sequence + 1),
        cond: Preconditions::None,
        memo: Memo::None,
        operations: vec![Operation {
            source_account: None,
            body: OperationBody::BumpFootprintExpiration(BumpFootprintExpirationOp {
                ext: ExtensionPoint::V0,
                ledgers_to_expire: min_ledgers_to_live,
            }),
        }]
        .try_into().unwrap(), // TODO: error handling

        ext: TransactionExt::V1(SorobanTransactionData {
            ext: ExtensionPoint::V0,
            resources: SorobanResources {
                footprint: LedgerFootprint {
                    read_only: parsed_keys.try_into().unwrap(), // TODO: error handling
                    read_write: vec![].try_into().unwrap(), // TODO: error handling
                },
                instructions: 0,
                read_bytes: 0,
                write_bytes: 0,
            },
            refundable_fee: 0,
        }),
    }
} 

/// Builds a bump operation for every contract instance in a single transaction
pub async fn bump_tx(target: Target, public: [u8; 32], contracts: Option<Vec<String>>, wasms: Option<Vec<String>>, sequence: i64, min_ledgers_to_live: u32) -> Transaction {
    let parsed_keys = match target {
        Target::Instance => build_instance_parsed_keys(contracts).await,
        Target::Code => build_code_parsed_keys(contracts, wasms).await
    };

    build_bump_tx(public, sequence, parsed_keys, min_ledgers_to_live)
}

fn build_restore_tx(public: [u8; 32], sequence: i64, parsed_keys: Vec<LedgerKey>) -> Transaction {
    let tx = Transaction {
        source_account: MuxedAccount::Ed25519(Uint256(public)),
        fee: DEFAULT_FEE,
        seq_num: SequenceNumber(sequence + 1),
        cond: Preconditions::None,
        memo: Memo::None,
        operations: vec![Operation {
            source_account: None,
            body: OperationBody::RestoreFootprint(RestoreFootprintOp {
                ext: ExtensionPoint::V0,
            }),
        }]
        .try_into().unwrap(),
        ext: TransactionExt::V1(SorobanTransactionData {
            ext: ExtensionPoint::V0,
            resources: SorobanResources {
                footprint: LedgerFootprint {
                    read_only: vec![].try_into().unwrap(),
                    read_write: parsed_keys.try_into().unwrap(),
                },
                instructions: 0,
                read_bytes: 0,
                write_bytes: 0,
            },
            refundable_fee: 0,
        }),
    };

    tx
} 

pub async fn restore_contract_instance_tx(target: Target, public: [u8; 32], contracts: Option<Vec<String>>, wasms: Option<Vec<String>>, sequence: i64) -> Transaction {
    let parsed_keys = match target {
        Target::Instance => build_instance_parsed_keys(contracts).await,
        Target::Code => build_code_parsed_keys(contracts, wasms).await
    };

    build_restore_tx(public, sequence, parsed_keys)
}

pub fn get_client(base_url: &str) -> Client {
    Client::new(base_url).unwrap()
}

async fn get_contract_wasm_hash(contract_id: [u8; 32]) -> Result<[u8; 32], reqwest::Error> {
    let req_client = reqwest::Client::new();
    let api_url = "https://rpc-futurenet.stellar.org:443";

    let key_xdr = LedgerKey::ContractData(LedgerKeyContractData { 
        contract: ScAddress::Contract(Hash(contract_id)),
        durability: ContractDataDurability::Persistent,
        key: ScVal::LedgerKeyContractInstance,
    }).to_xdr_base64().unwrap();

    let params = json!([
        key_xdr
    ]);

    let response = req_client.post(api_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getLedgerEntry",
            "params": params
        }))
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;

    let xdr_result = response_json["result"]["xdr"].as_str();
    if let Some(xdr) = xdr_result {
        let ledger_entry_data = LedgerEntryData::from_xdr_base64(xdr);
        
        match ledger_entry_data.unwrap() {
            LedgerEntryData::ContractData(data) => {
                match data.val {
                    ScVal::ContractInstance(instance) => {
                        match instance.executable {
                            ContractExecutable::Wasm(hash) => Ok(hash.0),
                            _ => panic!()
                        }
                    },
                    _ => panic!()
                }
            }

            _ => panic!()
        }
    } else {
        panic!()
    }
}


async fn build_code_parsed_keys(contracts: Option<Vec<String>>, wasms: Option<Vec<String>>) -> Vec<LedgerKey> {
    let mut contract_hashes = Vec::new();
            
    if let Some(contracts) = contracts {
        let mut contract_ids = Vec::new();
        for contract in contracts {
            let bytes = stellar_strkey::Contract::from_string(&contract).unwrap().0;
            contract_ids.push(bytes);
        }

        for contract in contract_ids {
            let instance = get_contract_wasm_hash(contract).await.unwrap();
            contract_hashes.push(instance)
        }
    }

    if let Some(hashes) = wasms {
        for hex_hash in hashes {
            let bytes_result = Vec::<u8>::from_hex(hex_hash);

            let hash = match bytes_result {
                Ok(bytes) => {
                    if bytes.len() == 32 {
                        let mut hash_array: [u8; 32] = [0; 32];
                        hash_array.copy_from_slice(&bytes);

                        hash_array
                    } else {
                        panic!("Hex string doesn't represent a 32-byte hash");
                    }
                }
                Err(_) => {
                    panic!("Invalid hex string");
                }
            };
            contract_hashes.push(hash.into());
        }
    }

    let mut parsed_keys = Vec::new();

    for contract in contract_hashes {
        let parsed_key = LedgerKey::ContractCode(LedgerKeyContractCode {
            hash: Hash(contract),
        });

        parsed_keys.push(parsed_key);
    }

    parsed_keys
}

async fn build_instance_parsed_keys(contracts: Option<Vec<String>>) -> Vec<LedgerKey> {
    let mut contract_ids = Vec::new();
    for contract in contracts.unwrap() {
        let bytes = stellar_strkey::Contract::from_string(&contract).unwrap().0;
        contract_ids.push(bytes);
    }
    let key = ScVal::LedgerKeyContractInstance;
    let mut parsed_keys = Vec::new();

    for contract in contract_ids {
        let parsed_key = LedgerKey::ContractData(LedgerKeyContractData {
            contract: ScAddress::Contract(Hash(contract)),
            durability: ContractDataDurability::Persistent,
            key: key.clone()
        });

        parsed_keys.push(parsed_key);
    }

    parsed_keys
}
