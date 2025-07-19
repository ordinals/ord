
from bitcointx.core import COutPoint, CTxIn, CTxOut, CMutableTransaction, lx
from bitcointx.wallet import CBitcoinSecret, P2PKHBitcoinAddress
from bitcointx.core.psbt import PartiallySignedTransaction
from bitcointx import SelectParams
from bitcointx.core.script import CScript, OP_DUP, OP_HASH160, OP_EQUALVERIFY, OP_CHECKSIG
from bitcointx.core import SignatureHash, SIGHASH_SINGLE, SIGHASH_ANYONECANPAY, SIGHASH_ALL

SelectParams('testnet')

# Private keys (WIF format)
taker_dummy_priv = CBitcoinSecret('cTakerDummyPrivateKeyWIF...')
taker_payment_priv = CBitcoinSecret('cTakerPaymentPrivateKeyWIF...')
maker_priv = CBitcoinSecret('cMakerInscriptionPrivateKeyWIF...')

# Address Scripts
dummy_address = P2PKHBitcoinAddress.from_pubkey(taker_dummy_priv.pub)
payment_address = P2PKHBitcoinAddress.from_pubkey(taker_payment_priv.pub)
maker_address = P2PKHBitcoinAddress.from_pubkey(maker_priv.pub)

# Transaction Inputs
# Replace the txids below with actual values

inputs = [
    CTxIn(COutPoint(lx('aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'), 0)),  # Dummy
    CTxIn(COutPoint(lx('bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'), 1)),  # Inscription
    CTxIn(COutPoint(lx('cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc'), 0))   # Payment
]

# Transaction Outputs
outputs = [
    CTxOut(500, dummy_address.to_scriptPubKey()),
    CTxOut(1, dummy_address.to_scriptPubKey()),
    CTxOut(100000, maker_address.to_scriptPubKey())
]

# Create Transaction and PSBT
transaction = CMutableTransaction(inputs, outputs)
psbt = PartiallySignedTransaction.from_tx(transaction)

# Maker signs Input 1 (inscription) with SIGHASH_SINGLE | ANYONECANPAY
sighash_flags = SIGHASH_SINGLE | SIGHASH_ANYONECANPAY
sighash = SignatureHash(maker_address.to_scriptPubKey(), transaction, 1, sighash_flags)
maker_sig = maker_priv.sign(sighash) + bytes([sighash_flags])
psbt.inputs[1].partial_sigs[maker_priv.pub] = maker_sig

# Taker signs Inputs 0 and 2 with SIGHASH_ALL
for idx, priv in zip([0, 2], [taker_dummy_priv, taker_payment_priv]):
    sighash = SignatureHash(dummy_address.to_scriptPubKey(), transaction, idx, SIGHASH_ALL)
    sig = priv.sign(sighash) + bytes([SIGHASH_ALL])
    psbt.inputs[idx].partial_sigs[priv.pub] = sig

# Final Transaction
final_tx = psbt.to_tx()
final_tx_hex = final_tx.serialize().hex()

print('Final transaction hex:')
print(final_tx_hex)
