from bitcointx.core import COutPoint, CTxIn, CTxOut, CMutableTransaction, lx
from bitcointx.wallet import CBitcoinSecret, P2PKHBitcoinAddress
from bitcointx.core.psbt import PartiallySignedTransaction
from bitcointx import SelectParams
from bitcointx.core.script import OP_DUP, OP_HASH160, OP_EQUALVERIFY, OP_CHECKSIG
from bitcointx.core import SignatureHash, SIGHASH_SINGLE, SIGHASH_ANYONECANPAY, SIGHASH_ALL

print("[INFO] Setting Bitcoin network to testnet")
SelectParams('testnet')

# === Testnet Simulation Set 1 ===
try:
    print("[INFO] Loading private keys")
    taker_dummy_priv = CBitcoinSecret('cTakerDummyPrivateKeyWIF...')
    taker_payment_priv = CBitcoinSecret('cTakerPaymentPrivateKeyWIF...')
    maker_priv = CBitcoinSecret('cMakerInscriptionPrivateKeyWIF...')
except Exception as e:
    print("[ERROR] Failed to load keys:", e)
    raise

try:
    print("[INFO] Deriving addresses")
    dummy_address = P2PKHBitcoinAddress.from_pubkey(taker_dummy_priv.pub)
    payment_address = P2PKHBitcoinAddress.from_pubkey(taker_payment_priv.pub)
    maker_address = P2PKHBitcoinAddress.from_pubkey(maker_priv.pub)
except Exception as e:
    print("[ERROR] Failed to derive addresses:", e)
    raise

print("[INFO] Creating transaction inputs")
inputs = [
    CTxIn(COutPoint(lx('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'), 0)),
    CTxIn(COutPoint(lx('bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'), 1)),
    CTxIn(COutPoint(lx('cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc'), 0))
]

print("[INFO] Creating transaction outputs")
outputs = [
    CTxOut(500, dummy_address.to_scriptPubKey()),
    CTxOut(1, dummy_address.to_scriptPubKey()),
    CTxOut(100000, maker_address.to_scriptPubKey())
]

print("[INFO] Building unsigned transaction")
transaction = CMutableTransaction(inputs, outputs)
psbt = PartiallySignedTransaction.from_tx(transaction)

try:
    print("[INFO] Maker signing input 1 with SIGHASH_SINGLE | ANYONECANPAY")
    sighash_flags = SIGHASH_SINGLE | SIGHASH_ANYONECANPAY
    sighash = SignatureHash(maker_address.to_scriptPubKey(), transaction, 1, sighash_flags)
    maker_sig = maker_priv.sign(sighash) + bytes([sighash_flags])
    psbt.inputs[1].partial_sigs[maker_priv.pub] = maker_sig
    print(f"[DEBUG] Maker signature (hex): {maker_sig.hex()}")
except Exception as e:
    print("[ERROR] Maker failed to sign input 1:", e)
    raise

print("[INFO] Taker signing inputs 0 and 2 with SIGHASH_ALL")
for idx, priv in zip([0, 2], [taker_dummy_priv, taker_payment_priv]):
    try:
        print(f"[DEBUG] Signing input {idx}")
        sighash = SignatureHash(dummy_address.to_scriptPubKey(), transaction, idx, SIGHASH_ALL)
        sig = priv.sign(sighash) + bytes([SIGHASH_ALL])
        psbt.inputs[idx].partial_sigs[priv.pub] = sig
        print(f"[DEBUG] Taker input {idx} signature (hex): {sig.hex()}")
    except Exception as e:
        print(f"[ERROR] Failed to sign input {idx}:", e)
        raise

print("[INFO] Serializing PSBT to base64")
try:
    psbt_base64 = psbt.to_base64()
    print("[RESULT] Final PSBT (base64):")
    print(psbt_base64)
except Exception as e:
    print("[ERROR] Failed to serialize PSBT:", e)
    raise

print("[INFO] Finalizing transaction")
try:
    final_tx = psbt.to_tx()
    final_tx_hex = final_tx.serialize().hex()
    print("[RESULT] Simulated TX Hex:")
    print(final_tx_hex)
except Exception as e:
    print("[ERROR] Failed to finalize transaction:", e)
    raise

# === Testnet Simulation Set 2 ===
# Replace keys with:
# cSTestnetDummy1WifKey11111111111111111111111111111111
# cSTestnetPayment1WifKey1111111111111111111111111111
# cSTestnetMaker1WifKey1111111111111111111111111111111111111

# === Testnet Simulation Set 3 ===
# Replace keys with:
# cPaJzs6nYBt8F9pZAcZUZQTn8f2PNvXb7tJ1pLjKGHpM6RKmLg88
# cT1x5JDkve97nRV8RJ5yPWBsoFEJ3bikx4zWxan7vgadFPmRzvBR
# cPLNdPbCMZcYHnbEEoHzpgacCTspPHJz3VnTH6HSoGpgFzzUR8ry

# All follow the same PSBT pattern as described above.
