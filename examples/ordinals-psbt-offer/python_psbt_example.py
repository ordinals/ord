from bitcointx.core import COutPoint, CTxIn, CTxOut, CMutableTransaction, lx
from bitcointx.wallet import CBitcoinSecret, P2PKHBitcoinAddress
from bitcointx.core.psbt import PartiallySignedTransaction
from bitcointx import SelectParams
from bitcointx.core.script import OP_DUP, OP_HASH160, OP_EQUALVERIFY, OP_CHECKSIG
from bitcointx.core import SignatureHash, SIGHASH_SINGLE, SIGHASH_ANYONECANPAY, SIGHASH_ALL

SelectParams('testnet')

# === Testnet Simulation Set 1 ===
taker_dummy_priv = CBitcoinSecret('cTakerDummyPrivateKeyWIF...')
taker_payment_priv = CBitcoinSecret('cTakerPaymentPrivateKeyWIF...')
maker_priv = CBitcoinSecret('cMakerInscriptionPrivateKeyWIF...')

dummy_address = P2PKHBitcoinAddress.from_pubkey(taker_dummy_priv.pub)
payment_address = P2PKHBitcoinAddress.from_pubkey(taker_payment_priv.pub)
maker_address = P2PKHBitcoinAddress.from_pubkey(maker_priv.pub)

inputs = [
    CTxIn(COutPoint(lx('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'), 0)),
    CTxIn(COutPoint(lx('bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'), 1)),
    CTxIn(COutPoint(lx('cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc'), 0))
]

outputs = [
    CTxOut(500, dummy_address.to_scriptPubKey()),
    CTxOut(1, dummy_address.to_scriptPubKey()),
    CTxOut(100000, maker_address.to_scriptPubKey())
]

transaction = CMutableTransaction(inputs, outputs)
psbt = PartiallySignedTransaction.from_tx(transaction)

sighash_flags = SIGHASH_SINGLE | SIGHASH_ANYONECANPAY
sighash = SignatureHash(maker_address.to_scriptPubKey(), transaction, 1, sighash_flags)
maker_sig = maker_priv.sign(sighash) + bytes([sighash_flags])
psbt.inputs[1].partial_sigs[maker_priv.pub] = maker_sig

for idx, priv in zip([0, 2], [taker_dummy_priv, taker_payment_priv]):
    sighash = SignatureHash(dummy_address.to_scriptPubKey(), transaction, idx, SIGHASH_ALL)
    sig = priv.sign(sighash) + bytes([SIGHASH_ALL])
    psbt.inputs[idx].partial_sigs[priv.pub] = sig

final_tx = psbt.to_tx()
print("Simulated TX Hex:", final_tx.serialize().hex())

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
