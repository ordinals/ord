Burning
=======

Inscriptions may be burned by constructing a transaction that spends them to a
script pubkey beginning with `OP_RETURN`.

Sending inscriptions to a so-called "burn address" is not recognized by `ord`.

Burned inscriptions receive the "burned" charm, recognized with 🔥 on the
inscription's `/inscription` page.

When burning inscriptions, CBOR metadata may be included in a data push
immediately following the `OP_RETURN`.

Burn metadata is unstructured, having no meaning to the underlying protocol,
and should be human readable. It is displayed on the burned inscription's
`/inscription` page, in the same manner as inscription metadata, under the
heading "burn metadata".

Use it, if you feel like it, to commemorate the inscription, celebrate the
closing of a collection, or for whatever other purposes you so desire.

Data pushes after the first are currently ignored by `ord`. However, they may
be given future meaning by the protocol, and should not be used.
