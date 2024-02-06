start-regtest:
  bitcoind -regtest -txindex

load wallet="ord":
  bitcoin-cli -regtest -rpcwallet={{wallet}} loadwallet {{wallet}}

mine blocks="1" wallet="ord":
  #!/usr/bin/env bash
  set -euxo pipefail
  address="$(bitcoin-cli -regtest -rpcwallet={{wallet}} getrawchangeaddress "bech32m")"
  bitcoin-cli -regtest generatetoaddress {{blocks}} $address

dump wallet="ord":
  RUST_LOG=info cargo run -- --regtest wallet dump

deploy-preview domain="preview.inscribingatlantis.com" remote="InscribingAtlantis/ord" branch="master" chain="regtest":
  ssh root@{{domain}} 'mkdir -p deploy \
    && apt-get update --yes \
    && apt-get upgrade --yes \
    && apt-get install --yes git rsync'
  rsync -avz deploy/checkout root@{{domain}}:deploy/checkout
  ssh root@{{domain}} 'cd deploy && ./checkout {{branch}} {{remote}} {{chain}} {{domain}}'
