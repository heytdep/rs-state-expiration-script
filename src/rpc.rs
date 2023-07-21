use soroban_cli::rpc::Client;
use soroban_env_host::xdr::{Transaction, MuxedAccount, Uint256, SequenceNumber, Preconditions, Memo, Operation, OperationBody, BumpFootprintExpirationOp, ExtensionPoint, TransactionExt, SorobanTransactionData, SorobanResources, LedgerFootprint, LedgerKey, LedgerKeyContractData, ScAddress, Hash, ContractEntryBodyType, ContractDataDurability, ScVal};

const DEFAULT_FEE: u32 = 100;

fn build_bump_tx(public: [u8; 32], sequence: i64, parsed_keys: Vec<LedgerKey>, min_ledgers_to_live: u32) -> Transaction {
    // let source = stellar_strkey::ed25519::PublicKey(public);

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
                extended_meta_data_size_bytes: 0,
            },
            refundable_fee: 0,
        }),
    }
} 

/// Builds a bump operation for every contract instance in a single transaction
pub fn bump_contract_instance_tx(public: [u8; 32], contract_ids: Vec<[u8; 32]>, sequence: i64, min_ledgers_to_live: u32) -> Transaction {
    let key = ScVal::LedgerKeyContractInstance;
    let mut parsed_keys = Vec::new();

    for contract in contract_ids {
        let parsed_key = LedgerKey::ContractData(LedgerKeyContractData {
            contract: ScAddress::Contract(Hash(contract)),
            durability: ContractDataDurability::Persistent,
            body_type: ContractEntryBodyType::DataEntry,
            key: key.clone()
        });

        parsed_keys.push(parsed_key);
    }

    build_bump_tx(public, sequence, parsed_keys, min_ledgers_to_live)
}

pub fn get_client(base_url: &str) -> Client {
    Client::new(base_url).unwrap()
}
