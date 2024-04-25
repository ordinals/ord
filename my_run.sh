#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"
export RUST_LOG=debug
export DATABASE_URL="postgres://postgres:postgres@localhost/db_demo"
cargo run -- -t \
    --bitcoin-rpc-url http://192.168.1.253:18332 \
    --bitcoin-rpc-username mike  --bitcoin-rpc-password apd3g41pkl \
    --index-runes --index-transactions \
    --first-inscription-height=2583205 \
    --commit-interval 1 \
    --index ../supersats_testnet.redb \
    server --address 0.0.0.0 --http-port 8088 > ../supersats.log 2>&1