from bitcointx.core import (
    COutPoint, CTxIn, CTxOut, CMutableTransaction, lx
)
from bitcointx.wallet import CBitcoinSecret, P2PKHBitcoinAddress
from bitcointx.core.psbt import PartiallySignedTransaction
from bitcointx import SelectParams
from bitcointx.core.script import OP_DUP, OP_HASH160, OP_EQUALVERIFY, OP_CHECKSIG
from bitcointx.core import SignatureHash, SIGHASH_SINGLE, SIGHASH_ANYONECANPAY, SIGHASH_ALL

print("[INFO] Setting Bitcoin network to testnet")
SelectParams('testnet')

# === Load Keys ===
try:
    print("[INFO] Loading private keys")
    taker_dummy_priv = CBitcoinSecret('cTakerDummyPrivateKeyWIF...')
    taker_payment_priv = CBitcoinSecret('cTakerPaymentPrivateKeyWIF...')
    maker_priv = CBitcoinSecret('cMakerInscriptionPrivateKeyWIF...')
except Exception as e:
    print("[ERROR] Failed to load keys:", e)
    raise

# === Derive Addresses ===
try:
    print("[INFO] Deriving addresses")
    dummy_address = P2PKHBitcoinAddress.from_pubkey(taker_dummy_priv.pub)
    payment_address = P2PKHBitcoinAddress.from_pubkey(taker_payment_priv.pub)
    maker_address = P2PKHBitcoinAddress.from_pubkey(maker_priv.pub)
except Exception as e:
    print("[ERROR] Failed to derive addresses:", e)
    raise

# === Create Inputs ===
print("[INFO] Creating transaction inputs")
inputs = [
    CTxIn(COutPoint(lx('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'), 0)),
    CTxIn(COutPoint(lx('bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'), 1)),
    CTxIn(COutPoint(lx('cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc'), 0))
]

# === Create Outputs ===
print("[INFO] Creating transaction outputs")
outputs = [
    CTxOut(10000, dummy_address.to_scriptPubKey()),
    CTxOut(20000, payment_address.to_scriptPubKey()),
    CTxOut(30000, maker_address.to_scriptPubKey())
]

# === Assemble Transaction ===
print("[INFO] Assembling mutable transaction")
tx = CMutableTransaction(inputs, outputs)

# === Wrap in PSBT ===
print("[INFO] Wrapping transaction in PSBT")
psbt = PartiallySignedTransaction.from_tx(tx)

# === Show raw PSBT base64 ===
print("[RESULT] PSBT (Base64 Encoded):")
print(psbt.to_base64())
