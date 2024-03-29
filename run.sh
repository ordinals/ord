#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "$0" )" && pwd )"
mainnet() {
    RUST_LOG=info /usr/local/bin/ord \
    --bitcoin-rpc-url ${ORD_BITCOIN_RPC_URL} \
    --index-runes --index-transactions \
    --commit-interval ${ORD_COMMIT_INTERVAL} \
    --index /opt/data/${ORD_INDEX_FILE} \
    server --address 0.0.0.0 --http-port 8088 > /opt/logs/${ORD_LOGFILE} 2>&1
}
#For docker
testnet() {
    RUST_LOG=info /usr/local/bin/ord -t \
    --bitcoin-rpc-url ${ORD_BITCOIN_RPC_URL} \
    --index-runes --index-transactions \
    --commit-interval ${ORD_COMMIT_INTERVAL} \
    --index /opt/data/runebeta_index_docker.redb \
    server --address 0.0.0.0 --http-port 8088 > /opt/logs/${ORD_LOGFILE} 2>&1
}

$@