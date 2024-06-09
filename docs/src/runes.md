Runes
=====

Runes allow Bitcoin transactions to etch, mint, and transfer Bitcoin-native
digital commodities.

Whereas every inscription is unique, every unit of a rune is the same. They are
interchangeable tokens, fit for a variety of purposes.

Runestones
----------

Rune protocol messages, called runestones, are stored in Bitcoin transaction
outputs.

A runestone output's script pubkey begins with an `OP_RETURN`, followed by
`OP_13`, followed by zero or more data pushes. These data pushes are
concatenated and decoded into a sequence of 128-bit integers, and finally
parsed into a runestone.

A transaction may have at most one runestone.

A runestone may etch a new rune, mint an existing rune, and transfer runes from
a transaction's inputs to its outputs.

A transaction output may hold balances of any number of runes.

Runes are identified by IDs, which consist of the block in which a rune was
etched and the index of the etching transaction within that block, represented
in text as `BLOCK:TX`. For example, the ID of the rune etched in the 20th
transaction of the 500th block is `500:20`.

Etching
-------

Runes come into existence by being etched. Etching creates a rune and sets its
properties. Once set, these properties are immutable, even to its etcher.

### Name

Names consist of the letters A through Z and are between one and twenty-six
letters long. For example `UNCOMMONGOODS` is a rune name.

Names may contain spacers, represented as bullets, to aid readability.
`UNCOMMONGOODS` might be etched as `UNCOMMON•GOODS`.

The uniqueness of a name does not depend on spacers. Thus, a rune may not be
etched with the same sequence of letters as an existing rune, even if it has
different spacers.

Spacers can only be placed between two letters. Finally, spacers do not
count towards the letter count.

### Divisibility

A rune's divisibility is how finely it may be divided into its atomic units.
Divisibility is expressed as the number of digits permissible after the decimal
point in an amount of runes. A rune with divisibility 0 may not be divided. A
unit of a rune with divisibility 1 may be divided into ten sub-units, a rune
with divisibility 2 may be divided into a hundred, and so on.

### Symbol

A rune's currency symbol is a single Unicode code point, for example `$`, `⧉`,
or `🧿`, displayed after quantities of that rune.

101 atomic units of a rune with divisibility 2 and symbol `🧿` would be
rendered as `1.01 🧿`.

If a rune does not have a symbol, the generic currency sign `¤`, also called a
scarab, should be used.

### Premine

The etcher of a rune may optionally allocate to themselves units of the rune
being etched. This allocation is called a premine.

### Terms

A rune may have an open mint, allowing anyone to create and allocate units of
that rune for themselves. An open mint is subject to terms, which are set upon
etching.

A mint is open while all terms of the mint are satisfied, and closed when any
of them are not. For example, a mint may be limited to a starting height, an
ending height, and a cap, and will be open between the starting height and
ending height, or until the cap is reached, whichever comes first.

#### Cap

The number of times a rune may be minted is its cap. A mint is closed once the
cap is reached.

#### Amount

Each mint transaction creates a fixed amount of new units of a rune.

#### Start Height

A mint is open starting in the block with the given start height.

#### End Height

A rune may not be minted in or after the block with the given end height.

#### Start Offset

A mint is open starting in the block whose height is equal to the start offset
plus the height of the block in which the rune was etched.

#### End Offset

A rune may not be minted in or after the block whose height is equal to the end
offset plus the height of the block in which the rune was etched.

Minting
-------

While a rune's mint is open, anyone may create a mint transaction that creates
a fixed amount of new units of that rune, subject to the terms of the mint.

Transferring
------------

When transaction inputs contain runes, or new runes are created by a premine or
mint, those runes are transferred to that transaction's outputs. A
transaction's runestone may change how input runes transfer to outputs.

### Edicts

A runestone may contain any number of edicts. Edicts consist of a rune ID, an
amount, and an output number. Edicts are processed in order, allocating
unallocated runes to outputs.

### Pointer

After all edicts are processed, remaining unallocated runes are transferred to
the transaction's first non-`OP_RETURN` output. A runestone may optionally
contain a pointer that specifies an alternative default output.

### Burning

Runes may be burned by transferring them to an `OP_RETURN` output with an edict
or pointer.

Cenotaphs
---------

Runestones may be malformed for a number of reasons, including non-pushdata
opcodes in the runestone `OP_RETURN`, invalid varints, or unrecognized
runestone fields.

Malformed runestones are termed
[cenotaphs](https://en.wikipedia.org/wiki/Cenotaph).

Runes input to a transaction with a cenotaph are burned. Runes etched in a
transaction with a cenotaph are set as unmintable. Mints in a transaction with
a cenotaph count towards the mint cap, but the minted runes are burned.

Cenotaphs are an upgrade mechanism, allowing runestones to be given new
semantics that change how runes are created and transferred, while not
misleading unupgraded clients as to the location of those runes, as unupgraded
clients will see those runes as having been burned.
