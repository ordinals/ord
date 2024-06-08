#!/usr/bin/env bash

set -eou pipefail

echo "["
bitcoin-cli -testnet -datadir=/var/lib/bitcoind getblock "00000000000000c9804c9d3b788fff4ffd096edd8c5709e349af792007003fb0" 1 | jq -r '.tx[]' | while read txid; do
  ord --data-dir /var/lib/ord/ --bitcoin-data-dir /var/lib/bitcoind --testnet decode --txid $txid --compact
  echo ","
done
echo "]"
