#!/usr/bin/env bash

set -euo pipefail

# Codex f303 Blocktest Realizer Script
#
# This helper regenerates the private key and address used by the
# codex_f303_blocktest genesis configuration, initializes a local geth
# datadir, and prepares it for mining so transactions can be tested.

printf '[INFO] Generating private key...\n'
PRIV=$(openssl rand -hex 32)
printf '[INFO] Generated private key: %s\n' "$PRIV"

printf '[INFO] Deriving Ethereum address...\n'
ADDR=$(PRIV="$PRIV" python3 - <<'PY')
from eth_account import Account
import os

priv = os.environ["PRIV"]
acct = Account.from_key(bytes.fromhex(priv))
print(acct.address)
PY
printf '[INFO] Derived address: %s\n' "$ADDR"

printf '[INFO] Patching genesis file...\n'
cp codex_f303_blocktest.json codex_f303_blocktest_patched.json
sed -i "s/0x90f8bf6A479f320ead074411a4B0e7944Ea8c9C1/$ADDR/" codex_f303_blocktest_patched.json
printf '[INFO] Patched genesis file with new address: %s\n' "$ADDR"

printf '[INFO] Initializing geth datadir...\n'
DATADIR=${DATADIR:-$HOME/codex-geth}
mkdir -p "$DATADIR"
geth --datadir "$DATADIR" init codex_f303_blocktest_patched.json

printf '[INFO] Importing private key into keystore...\n'
KEYFILE=$(mktemp)
printf '%s' "$PRIV" > "$KEYFILE"
geth --datadir "$DATADIR" account import "$KEYFILE"
rm "$KEYFILE"

cat <<EOF_MSG
[INFO] All set. To start geth, run:

geth --datadir "$DATADIR" \\
  --networkid 12345 \\
  --http --http.addr 0.0.0.0 --http.port 8545 \\
  --http.api personal,eth,net,web3,miner,txpool \\
  --allow-insecure-unlock \\
  --nodiscover \\
  --miner.threads 1 \\
  console

Then in the geth console:

> personal.unlockAccount("$ADDR", "<your-passphrase>", 300)
> eth.sendTransaction({from:"$ADDR", to:"0xAnotherAddr", value: web3.toWei(1,"ether")})
EOF_MSG
