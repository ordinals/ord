import json
import sys
from bitcointx.wallet import CBitcoinSecret
from bitcointx.core import lx
from bitcointx.core.psbt import PSBT
from bitcointx import SelectParams

SelectParams("testnet")

with open(sys.argv[1], 'r') as f:
    offer = json.load(f)

psbt = PSBT.from_base64(offer['maker']['partial_psbt'])

# Dummy key & payment key passed in as args
dummy_priv = CBitcoinSecret(sys.argv[2])
payment_priv = CBitcoinSecret(sys.argv[3])

# Add taker's inputs & outputs (placeholder logic)
# psbt.inputs.append(...)
# psbt.outputs.append(...)

# Sign taker's inputs
# psbt.inputs[0].sign_with(dummy_priv)
# psbt.inputs[2].sign_with(payment_priv)

final_tx = psbt.to_tx()
print("Final TX Hex:", final_tx.serialize().hex())
