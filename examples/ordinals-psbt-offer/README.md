Offer to Sell Inscriptions / Rare Sats (Ordinals Style)

Objective:

The Maker wants to sell an inscription (rare sat) to the Taker, who pays cardinal sats.

Structure Overview:

Inputs      Who Provides    Purpose
A           Taker           Dummy UTXO to meet index rule
B           Maker           UTXO containing inscription/rare sat
C           Taker           Payment UTXO (cardinal sats)

Outputs     Who Receives    Purpose
X           Taker           Receives the inscription/rare sat
Y           Maker           Receives payment (cardinal sats)

Bitcoin Mechanics (Why This Works)

Ordinal Flow: First input to first output. Inputs and outputs order matters.
SIGHASH_SINGLE | ANYONECANPAY: Maker signs only their input (B) and output Y. This prevents the taker from changing what the maker receives.
Taker completes transaction: Adds inputs A, C and outputs X, Y.
Trustless: Maker cannot be cheated, taker cannot be cheated.

Example PSBT Flow (Detailed)

Maker's Actions (Prepares Offer)

Selects UTXO B containing the inscription.
Prepares output Y with their own payment address.
Signs only input B and output Y with SIGHASH_SINGLE | ANYONECANPAY.
Broadcasts this partial PSBT to the Taker.

Taker's Actions (Finalizes and Executes)

Provides:
Dummy input A (small-value UTXO, e.g., 600 sats)
Payment input C (cardinal UTXO, enough to cover the price)

Outputs:
X: Taker’s receiving address for the inscription.
Y: Maker’s payment address (already set by Maker)

Signs all their inputs and outputs with SIGHASH_ALL.
Broadcasts the finalized transaction.

Example Visualized (Numbers Simplified)

Inputs:
#0 A - 600 sats dummy (Taker)
#1 B - 1 sat inscription (Maker)
#2 C - 100,000 sats payment (Taker)

Outputs:
#0 X - 1 sat inscription to Taker
#1 Y - 100,000 sats to Maker
#2 Optional Change - 500 sats to Taker

Maker's PSBT:

Inputs:
#1 B (1 sat) inscription UTXO
Outputs:
#1 Y (100,000 sats) payment address (Maker)

Signatures:
SIGHASH_SINGLE | ANYONECANPAY on input B

Taker Finalizes PSBT:

Adds:
Input A
Input C
Outputs X and Change
Signs:
SIGHASH_ALL

Resulting Transaction (On-Chain)

Input    Output                 Sats
0        Change to Taker         500
1        Inscription to Taker      1
2        Payment to Maker     100,000

Why This is Safe (No Trust)

Maker cannot alter inputs A or C.
Taker cannot alter Maker’s output Y.
PSBT signatures prevent tampering.
Inscription lands on Taker’s address only.

Optional Refinements

Use PSBT v2 for cleaner change handling.
Dummy input can be recycled change UTXO.
Add metadata or commitments off-chain to reference the sale.

Summary Benefits

Taker gets the inscription.
Maker gets the payment.
No trust required.
Fully Bitcoin-native, ordinal-safe.

