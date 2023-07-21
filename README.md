# Instance Bumper script

The script is functional and very simple. Starting from a json file, `bump-settings.json` it will bump instance following the file's rules.

For example, this json setting:

```json
{
    "contracts": ["CBFKJ2F7U4S6OOILIERHXNCZVCBUT7QNXSWHSU5U7FYOCAV7NETKEYIK", "CCJKF5BKXBAF4OWVUUN2SI7AT7CGUITILFF4SP6WY6OUI4E5IIDEQRWU"],
    "min_ledgers_to_live": 200000,
    "rpc_url": "https://rpc-futurenet.stellar.org:443/",
    "network": "Test SDF Future Network ; October 2022"
}
```

Will bump the contract instance for the those two contracts by 20000 ledgers.

To run the script:

```
cargo run -- --secret $SECRET_KEY
```

To handle the bumping of persistent entries not in the contract's instance storage the script will also support a custom keys section in the bump settings that specifies the key and the contract to bump.


# Known limitations

There really isn't a known limitation, but one thing I'm not sure of is how large can the footprint be without exceeding parameters. Assuming that the bump operation writes to the ledgers, there may be a limit if there are more than 20 (I think) contracts or custom keys. In that situation the script can simply split the number of keys and perform two transactions.
