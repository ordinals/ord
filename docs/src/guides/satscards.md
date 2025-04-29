Satscard
========

[Satscards](https://satscard.com/) are cards which can be used to store
bitcoin, inscriptions, and runes.

Slots
-----

Each satscard has ten slots containing private keys with corresponding bitcoin
addresses.

Initially, all slots are sealed and the private keys are stored only the
satscard.

Slots can be unsealed, which allows the corresponding private key to be
extracted.

Unsealing is permanent. If a satscard is sealed, you can have some confidence
that private key is not known to anyone. That taking physical ownership of a
satscard makes you the sole owner of assets in any sealed slots.

Lifespan
--------

Satscards are expected to have a usable lifetime of ten years. Do not use
satscards for long-term storage of valuable assets.


Viewing
-------

When placed on a smartphone, the satscard transmits a URL, beginning with
`https://satscard.com/start` or `https://getsatscard.com/start`, depending on
when it was manufactured.

This URL contains a signature which can be used to recover the address of the
current slot. This signature is made over a random nonce, so it changes every
time the satscard is tapped, and provides some confidence that the satscard
contains the private key.

`ord` supports viewing the contents of a satscard by entering the full URL into
the `ord` explorer search bar, or the input field on the `/satscard` page.

For `ordinals.com`, this is
[ordinals.com/satscard](https://ordinals.com/satscard).

Unsealing
---------

Satscard slots can be unsealed and the private keys extracted using the `cktap`
binary, available in the
[coinkite-tap-proto](https://github.com/coinkite/coinkite-tap-proto)
repository.

Sweeping
--------

After a satscard slot is unsealed, all assets should be swept from that slot to
another wallet, as the private key can now be read via NFC.

`ord` does not yet support sweeping assets from other wallets, so assets will
need to be transferred manually.

Be careful, and good luck!
