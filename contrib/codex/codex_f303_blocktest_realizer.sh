#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
GENESIS_TEMPLATE="$SCRIPT_DIR/codex_f303_blocktest.json"
GENESIS_PATCHED="$SCRIPT_DIR/codex_f303_blocktest_patched.json"
DATADIR_DEFAULT="$HOME/codex-geth"
KEYFILE=""

cleanup() {
  rm -f "$GENESIS_PATCHED"
  if [[ -n "$KEYFILE" ]]; then
    rm -f "$KEYFILE"
  fi
}
trap cleanup EXIT

if [[ ! -f "$GENESIS_TEMPLATE" ]]; then
  echo "[ERROR] Genesis template not found: $GENESIS_TEMPLATE" >&2
  exit 1
fi

DATADIR="${DATADIR:-$DATADIR_DEFAULT}"

echo "[INFO] Step 1: Generating private key"
PRIV=$(openssl rand -hex 32)
echo "[INFO] Generated private key: $PRIV"

echo "[INFO] Step 2: Computing address from private key"
ADDR=$(PRIV="$PRIV" python3 - <<'PY'
import os
from eth_account import Account

priv = os.environ["PRIV"]
acct = Account.from_key(bytes.fromhex(priv))
print(acct.address)
PY
)
echo "[INFO] Derived address: $ADDR"

echo "[INFO] Step 3: Patching genesis with new address"
cp "$GENESIS_TEMPLATE" "$GENESIS_PATCHED"
sed -i "s/0x90f8bf6a479f320ead074411a4b0e7944ea8c9c1/${ADDR,,}/" "$GENESIS_PATCHED"
echo "[INFO] Patched genesis file written to $GENESIS_PATCHED"

echo "[INFO] Step 4: Initializing geth datadir at $DATADIR"
mkdir -p "$DATADIR"
geth --datadir "$DATADIR" init "$GENESIS_PATCHED"

echo "[INFO] Step 5: Importing private key into keystore"
KEYFILE=$(mktemp)
printf '%s' "$PRIV" > "$KEYFILE"
geth --datadir "$DATADIR" account import "$KEYFILE"
rm -f "$KEYFILE"
KEYFILE=""

echo "[INFO] All set. To start geth, run the following command:\n"
cat <<EOM
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
> eth.sendTransaction({from: "$ADDR", to: "0xAnotherAddr", value: web3.toWei(1, "ether")})
EOM
