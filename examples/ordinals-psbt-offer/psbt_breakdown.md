PSBT Breakdown for Ordinals Offer to Sell
Inputs
Dummy input (600 sats)
txid: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
vout: 0
Provided by taker
Inscription input (1 sat)
txid: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
vout: 1
Provided by maker
Payment input (100,000 sats)
txid: cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
vout: 0
Provided by taker
Outputs
500 sats (change back to taker)
1 sat (inscription to taker)
100,000 sats (payment to maker)
Signing Plan
Maker signs Input 1 with SIGHASH_SINGLE | ANYONECANPAY
Taker signs Inputs 0 and 2 with SIGHASH_ALL
PSBT Stages (Simplified)
Maker prepares PSBT:
Inputs: [Input 1]
Outputs: [Output 2]
Signs only Input 1 with SIGHASH_SINGLE | ANYONECANPAY
Exports PSBT to taker
Taker adds Inputs 0 and 2, Outputs 0 and 1
Signs Inputs 0 and 2 with SIGHASH_ALL
Final Transaction Structure (Simplified Example)
Inputs: 0. 600 sats dummy input 1. 1 sat inscription input 2. 100,000 sats payment input
Outputs: 0. 500 sats to taker (change) 1. 1 sat to taker (inscription) 2. 100,000 sats to maker (payment)
Final Broadcast Hex (example placeholder)
0200000003aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0000000000…
##########



from bitcointx.core import COutPoint, CTxIn, CTxOut, CMutableTransaction, lx from bitcointx.wallet import CBitcoinSecret, P2PKHBitcoinAddress from bitcointx.core.psbt import PartiallySignedTransaction from bitcointx import SelectParams from bitcointx.core.script import CScript, OP_DUP, OP_HASH160, OP_EQUALVERIFY, OP_CHECKSIG from bitcointx.core import SignatureHash, SIGHASH_SINGLE, SIGHASH_ANYONECANPAY, SIGHASH_ALL
SelectParams(‘testnet’)
Private keys (WIF format)
taker_dummy_priv = CBitcoinSecret(‘cTakerDummyPrivateKeyWIF…’) taker_payment_priv = CBitcoinSecret(‘cTakerPaymentPrivateKeyWIF…’) maker_priv = CBitcoinSecret(‘cMakerInscriptionPrivateKeyWIF…’)
Address Scripts
dummy_address = P2PKHBitcoinAddress.from_pubkey(taker_dummy_priv.pub) payment_address = P2PKHBitcoinAddress.from_pubkey(taker_payment_priv.pub) maker_address = P2PKHBitcoinAddress.from_pubkey(maker_priv.pub)
Transaction Inputs (testnet realistic txids)
inputs = [ CTxIn(COutPoint(lx(‘1a2b3c4d5e6f77889900aabbccddeeff11223344556677889900aabbccddeeff’), 0)), # Dummy CTxIn(COutPoint(lx(‘2b3c4d5e6f77889900aabbccddeeff11223344556677889900aabbccddeeffaa’), 1)), # Inscription CTxIn(COutPoint(lx(‘3c4d5e6f77889900aabbccddeeff11223344556677889900aabbccddeeffaabb’), 0)) # Payment]
Transaction Outputs
outputs = [ CTxOut(500, dummy_address.to_scriptPubKey()), CTxOut(1, dummy_address.to_scriptPubKey()), CTxOut(100000, maker_address.to_scriptPubKey())]
Create Transaction and PSBT
transaction = CMutableTransaction(inputs, outputs) psbt = PartiallySignedTransaction.from_tx(transaction)
Maker signs Input 1 (inscription) with SIGHASH_SINGLE | ANYONECANPAY
sighash_flags = SIGHASH_SINGLE | ANYONECANPAY sighash = SignatureHash(maker_address.to_scriptPubKey(), transaction, 1, sighash_flags) maker_sig = maker_priv.sign(sighash) + bytes([sighash_flags]) psbt.inputs[1].partial_sigs[maker_priv.pub] = maker_sig
Taker signs Inputs 0 and 2 with SIGHASH_ALL
for idx, priv in zip([0, 2], [taker_dummy_priv, taker_payment_priv]): sighash = SignatureHash(dummy_address.to_scriptPubKey(), transaction, idx, SIGHASH_ALL) sig = priv.sign(sighash) + bytes([SIGHASH_ALL]) psbt.inputs[idx].partial_sigs[priv.pub] = sig
Final Transaction
final_tx = psbt.to_tx() final_tx_hex = final_tx.serialize().hex()
print(‘Final transaction hex:’) print(final_tx_hex)

##########
PSBT Breakdown #2

from bitcointx.core import COutPoint, CTxIn, CTxOut, CMutableTransaction, lx from bitcointx.wallet import CBitcoinSecret, P2PKHBitcoinAddress from bitcointx.core.psbt import PartiallySignedTransaction from bitcointx import SelectParams from bitcointx.core.script import CScript, OP_DUP, OP_HASH160, OP_EQUALVERIFY, OP_CHECKSIG from bitcointx.core import SignatureHash, SIGHASH_SINGLE, SIGHASH_ANYONECANPAY, SIGHASH_ALL
SelectParams(‘testnet’)
Simulated testnet private keys (WIF)
taker_dummy_priv = CBitcoinSecret(‘cSTestnetDummy1WifKey11111111111111111111111111111111’) taker_payment_priv = CBitcoinSecret(‘cSTestnetPayment1WifKey1111111111111111111111111111’) maker_priv = CBitcoinSecret(‘cSTestnetMaker1WifKey1111111111111111111111111111111111111’)
Addresses from public keys
dummy_address = P2PKHBitcoinAddress.from_pubkey(taker_dummy_priv.pub) payment_address = P2PKHBitcoinAddress.from_pubkey(taker_payment_priv.pub) maker_address = P2PKHBitcoinAddress.from_pubkey(maker_priv.pub)
Simulated transaction inputs (testnet realistic txids)
inputs = [ CTxIn(COutPoint(lx(‘1a2b3c4d5e6f77889900aabbccddeeff11223344556677889900aabbccddeeff’), 0)), # Dummy input CTxIn(COutPoint(lx(‘2b3c4d5e6f77889900aabbccddeeff11223344556677889900aabbccddeeffaa’), 1)), # Inscription input CTxIn(COutPoint(lx(‘3c4d5e6f77889900aabbccddeeff11223344556677889900aabbccddeeffaabb’), 0)) # Payment input]
Transaction outputs
outputs = [ CTxOut(500, dummy_address.to_scriptPubKey()), CTxOut(1, dummy_address.to_scriptPubKey()), CTxOut(100000, maker_address.to_scriptPubKey())]
Create transaction and PSBT
transaction = CMutableTransaction(inputs, outputs) psbt = PartiallySignedTransaction.from_tx(transaction)
Maker signs Input 1 (inscription) with SIGHASH_SINGLE | ANYONECANPAY
sighash_flags = SIGHASH_SINGLE | ANYONECANPAY sighash = SignatureHash(maker_address.to_scriptPubKey(), transaction, 1, sighash_flags) maker_sig = maker_priv.sign(sighash) + bytes([sighash_flags]) psbt.inputs[1].partial_sigs[maker_priv.pub] = maker_sig
Taker signs Inputs 0 and 2 with SIGHASH_ALL
for idx, priv in zip([0, 2], [taker_dummy_priv, taker_payment_priv]): sighash = SignatureHash(dummy_address.to_scriptPubKey(), transaction, idx, SIGHASH_ALL) sig = priv.sign(sighash) + bytes([SIGHASH_ALL]) psbt.inputs[idx].partial_sigs[priv.pub] = sig
Final transaction
final_tx = psbt.to_tx() final_tx_hex = final_tx.serialize().hex()
Simulate final transaction hex output
print(‘Simulated final transaction hex:’) print(final_tx_hex)
This hex would now be broadcast via testnet API or Bitcoin Core sendrawtransaction


#########
PSBT Breakdown #3

from bitcointx.core import COutPoint, CTxIn, CTxOut, CMutableTransaction, lx from bitcointx.wallet import CBitcoinSecret, P2PKHBitcoinAddress from bitcointx.core.psbt import PartiallySignedTransaction from bitcointx import SelectParams from bitcointx.core.script import CScript, OP_DUP, OP_HASH160, OP_EQUALVERIFY, OP_CHECKSIG from bitcointx.core import SignatureHash, SIGHASH_SINGLE, SIGHASH_ANYONECANPAY, SIGHASH_ALL
SelectParams(‘testnet’)
Simulated testnet private keys (WIF)
taker_dummy_priv = CBitcoinSecret(‘cPaJzs6nYBt8F9pZAcZUZQTn8f2PNvXb7tJ1pLjKGHpM6RKmLg88’) taker_payment_priv = CBitcoinSecret(‘cT1x5JDkve97nRV8RJ5yPWBsoFEJ3bikx4zWxan7vgadFPmRzvBR’) maker_priv = CBitcoinSecret(‘cPLNdPbCMZcYHnbEEoHzpgacCTspPHJz3VnTH6HSoGpgFzzUR8ry’)
Addresses from public keys
dummy_address = P2PKHBitcoinAddress.from_pubkey(taker_dummy_priv.pub) payment_address = P2PKHBitcoinAddress.from_pubkey(taker_payment_priv.pub) maker_address = P2PKHBitcoinAddress.from_pubkey(maker_priv.pub)
Simulated transaction inputs (testnet realistic txids)
inputs = [ CTxIn(COutPoint(lx(‘1a2b3c4d5e6f77889900aabbccddeeff11223344556677889900aabbccddeeff’), 0)), # Dummy input CTxIn(COutPoint(lx(‘2b3c4d5e6f77889900aabbccddeeff11223344556677889900aabbccddeeffaa’), 1)), # Inscription input CTxIn(COutPoint(lx(‘3c4d5e6f77889900aabbccddeeff11223344556677889900aabbccddeeffaabb’), 0)) # Payment input]
Transaction outputs
outputs = [ CTxOut(500, dummy_address.to_scriptPubKey()), CTxOut(1, dummy_address.to_scriptPubKey()), CTxOut(100000, maker_address.to_scriptPubKey())]
Create transaction and PSBT
transaction = CMutableTransaction(inputs, outputs) psbt = PartiallySignedTransaction.from_tx(transaction)
Maker signs Input 1 (inscription) with SIGHASH_SINGLE | ANYONECANPAY
sighash_flags = SIGHASH_SINGLE | ANYONECANPAY sighash = SignatureHash(maker_address.to_scriptPubKey(), transaction, 1, sighash_flags) maker_sig = maker_priv.sign(sighash) + bytes([sighash_flags]) psbt.inputs[1].partial_sigs[maker_priv.pub] = maker_sig
Taker signs Inputs 0 and 2 with SIGHASH_ALL
for idx, priv in zip([0, 2], [taker_dummy_priv, taker_payment_priv]): sighash = SignatureHash(dummy_address.to_scriptPubKey(), transaction, idx, SIGHASH_ALL) sig = priv.sign(sighash) + bytes([SIGHASH_ALL]) psbt.inputs[idx].partial_sigs[priv.pub] = sig
Final transaction
final_tx = psbt.to_tx() final_tx_hex = final_tx.serialize().hex()
Simulate final transaction hex output
print(‘Simulated final transaction hex:’) print(final_tx_hex)
This hex would now be broadcast via testnet API or Bitcoin Core sendrawtransaction

