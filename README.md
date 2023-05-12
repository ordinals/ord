# Ordgrid 

An indexer for the Ordgrid protocol based on the original ord client. Just swap `ord index` for `ord ordgrid --redis_uri=<your_redis_endpoint>` and you're good to go.

## Development

Assuming a bitcoin node is running

    bitcoind -regtest -txindex=1 -datadir=~/dev/_databases/bitcoin_regtest -rpccookiefile=~/dev/_databases/bitcoin_regtest/cookie
    
    cargo run -- --regtest --data-dir="~/dev/_databases/ord_regtest" --cookie-file="~/dev/_databases/bitcoin_regtest/cookie" --reindex="true" ordgrid


## Build

rustup target add x86_64-apple-darwin

    cargo build --release

The default location for the `ord` binary once built is `./target/release/ord`.

`ord` requires `rustc` version 1.67.0 or later. Run `rustc --version` to ensure you have this version. Run `rustup update` to get the latest stable release.

## Testing

Run a bitcoin using the regtest network (see more how to use regtest here: https://gist.github.com/System-Glitch/cb4e87bf1ae3fec9925725bb3ebe223a)

    bitcoind -regtest -txindex=1 -datadir=/Volumes/1TB SSD/bitcoin_regtest -rpccookiefile=./cookie
    bitcoind -regtest -txindex=1 -datadir=~/dev/_databases/bitcoin_regtest -rpccookiefile=./cookie


Run ord client against the regtest network

    ord --regtest --data-dir=/Volumes/1TB SSD/ord_regtest --cookie-file=/Volumes/1TB SSD/bitcoin_regtest/cookie ordgrid
    ord --regtest --datadir=~/dev/_databases/ord_regtest --cookie-file=~/dev/_databases/bitcoin_regtest/cookie ordgrid

## Production

    bitcoind -txindex=1 -datadir=/Volumes/1TB SSD/bitcoin -rpccookiefile=./cookie
    ord --data-dir=/Volumes/1TB SSD/ord --cookie-file=/Volumes/1TB SSD/bitcoin/cookie ordgrid
## Syncing

`ord` requires a synced `bitcoind` node with `-txindex` to build the index of
satoshi locations. `ord` communicates with `bitcoind` via RPC.

If `bitcoind` is run locally by the same user, without additional
configuration, `ord` should find it automatically by reading the `.cookie` file
from `bitcoind`'s datadir, and connecting using the default RPC port.

If `bitcoind` is not on mainnet, is not run by the same user, has a non-default
datadir, or a non-default port, you'll need to pass additional flags to `ord`.
See `ord --help` for details.

## Logging

`ord` uses [env_logger](https://docs.rs/env_logger/latest/env_logger/). Set the
`RUST_LOG` environment variable in order to turn on logging. For example, run
the server and show `info`-level log messages and above:

```
$ RUST_LOG=info cargo run server
```

## Populating the blockchain

Create a wallet
    
    bitcoin-cli -regtest -rpccookiefile=~/dev/_databases/bitcoin_regtest/cookie createwallet "ordgrid_wallet"

Generate a new address (bcrt1qth99v0thsgl8acvnqalj8r2sdy26cqqhcv88mp)

    bitcoin-cli -regtest -rpccookiefile=~/dev/_databases/bitcoin_regtest/cookie  -rpcwallet=ordgrid_wallet_2 getnewaddress

Load the wallet

    bitcoin-cli -regtest -rpccookiefile=~/dev/_databases/bitcoin_regtest/cookie -rpcwallet=ordgrid_wallet_2 loadwallet "ordgrid_wallet_2"

Generate some blocks (we need more then 100 blocks to be able to transfer funds)

    bitcoin-cli -regtest -rpccookiefile=~/dev/_databases/bitcoin_regtest/cookie -rpcwallet=ordgrid_wallet_2 -generate 101

FIX: set the transaction fee

    bitcoin-cli -chain=regtest -rpccookiefile=~/dev/_databases/bitcoin_regtest/cookie -rpcwallet=ordgrid_wallet_2 settxfee 0.1

Transfer funds to the ord wallet address

    bitcoin-cli -regtest -rpccookiefile=~/dev/_databases/bitcoin_regtest/cookie sendtoaddress bcrt1p5a4gst0mcfjmzrqgth3grfvdhr5z4f2d5f7jmzcekwwsseelc53qw33xur 1

## Ord wallet

Inscribe 

    cargo run -- --regtest --data-dir="~/dev/_databases/ord_regtest" --cookie-file="~/dev/_databases/bitcoin_regtest/cookie" wallet inscribe --fee-rate 1 ./inscriptions/inscription_1_11.txt

## Server

cargo run -- --regtest --data-dir="~/dev/_databases/ord_regtest" --cookie-file="~/dev/_databases/bitcoin_regtest/cookie" server