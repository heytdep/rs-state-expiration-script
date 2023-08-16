# State expiration manager script

The script is functional and very simple. Starting from a json file, `bump-settings.json` it will deal with state expiration following the file's rules.

For example, this json setting:

```json
{
    "contracts": [
        "CDMQVP6T3OOPN34K4TCIGTVWG3DIAD45H6MODYRIXPR7G3BJ7OLCDEZF",
        "CDWSPRPIOECUHNN6RE4FKSSDOSA2QQ2SG5N5BMLDHI5FRFBIMO7YA3TY",
    ],
    "hashes": [
        "d690aa205ee16c27206b937142cd3e0a8a8c55572ce820d8e897f96d577c9251"
    ],
    "min_ledgers_to_live": 200000,
    "rpc_url": "https://rpc-futurenet.stellar.org:443/",
    "network": "Test SDF Future Network ; October 2022"
}
```

### Bump contracts instance

Running with the following command will bump the instance of the two specified contracts by 200000 ledgers:

```
cargo run -- --secret $SECRET_KEY --action Bump --target Instance
```

### Bump contracts code

Running with the following command will bump the contract code for both the code in the contracts and also the wasm hashes specified in `hashes`:

```
cargo run -- --secret $SECRET_KEY --action Bump --target Code
```

### Restore contracts instance
Running with the following command will restore the instance of the two specified contracts:

```
cargo run -- --secret $SECRET_KEY --action Restore --target Instance
```

### Restore contracts code
Running with the following command will restore the code of the two specified contracts along with the two wasm hashes:

```
cargo run -- --secret $SECRET_KEY --action Restore --target Code
```


## TODOs
- To handle the bumping of persistent entries not in the contract's instance storage the script will also support a custom keys section in the bump settings that specifies the key and the contract to bump.


# Known limitations

There really isn't a known limitation, but one thing I'm not sure of is how large can the footprint be without exceeding parameters. Assuming that the bump operation writes to the ledgers, there may be a limit if there are more than 20 (I think) contracts or custom keys. In that situation the script can simply split the number of keys and perform two transactions.
