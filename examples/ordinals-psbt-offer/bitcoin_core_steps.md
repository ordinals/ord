Bitcoin Core Manual Steps for Ordinals Trustless Offer to Sell
Step-by-step using Bitcoin Core CLI
1. Maker Creates Initial PSBT
createpsbt \
  '[{"txid":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","vout":1}]' \
  '[{"address":"maker_receive_payment_address","amount":0.001}]'
2. Maker Signs Input with Special Sighash Flags
utxoupdatepsbt (PSBT_OUTPUT_FROM_ABOVE)

signpsbtwithkey \
  (UPDATED_PSBT_FROM_ABOVE) \
  '["maker_private_key_WIF"]' \
  '{"bbbb...": {"scriptPubKey": "...", "amount": 0.00000001}}' \
  SIGHASH_SINGLE|SIGHASH_ANYONECANPAY
3. Taker Updates PSBT with Additional Inputs and Outputs
Taker adds: - Dummy Input (600 sats) - Payment Input (100,000 sats) - Outputs: - Change to self - Inscription to self - Payment to maker (already present)
utxoupdatepsbt (PSBT_FROM_MAKER_WITH_PARTIAL_SIG)
4. Taker Signs with SIGHASH_ALL
signpsbtwithkey \
  (UPDATED_PSBT_WITH_TAKER_INPUTS) \
  '["taker_dummy_key", "taker_payment_key"]' \
  '{"aaaa...": {"scriptPubKey": "...", "amount": 0.00000600}, "cccc...": {"scriptPubKey": "...", "amount": 0.00100000}}' \
  SIGHASH_ALL
5. Finalize PSBT and Extract Final Hex
finalizepsbt (SIGNED_PSBT)
getrawtransaction (from finalized PSBT)
6. Broadcast Final Transaction
sendrawtransaction (FINAL_HEX)
