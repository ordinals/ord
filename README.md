# Satoshi Ordinals

A scheme for assigning ordinal numbers to satoshis and tracking them across
transactions, and a command-line utility, `ord` for querying information about
ordinals.

Ordinal numbers can be used as an addressing scheme for NFTs. In such a scheme,
the NFT creator would create a message that assigned a new NFT Y to the satoshi
with ordinal X. The owner of the UTXO containing the satoshi with ordinal X
owns NFT Y, and can transfer that ownership to another person with a
transaction that sends ordinal Y to a UTXO that the new owner controls. The
current owner can sign a message proving that they own a given UTXO, which also
serves as proof of ownership of all the NFTs assigned to satoshis within that
UTXO.

See [the BIP](bip.mediawiki) for a comprehensive description of the assignment
and transfer algorithm.

## Index and Caveats

The `ord` command queries `bitcoind` for block data. Most commands require
`--rpc-url` and `--cookie-file`, which take the URL of a `bitcoind`'s JSON RPC
API and authentication cookie file respectively.

The index is stored in `index.redb`, and should not be concurrently modified
while an instance of `ord` is running, or used by two `ord` instances
simultaneously.

Currently, reorganizations are detected but not handled, the index is slow to
build and space-inefficient, and the full main chain has not yet been indexed.

## Numbering

Satoshis are assigned ordinal numbers in the order in which they are mined.
Ordinals start at 0, for the first satoshi of the genesis block, and end with
2099999997689999, the only satoshi mined in block 6929999, the last
subsidy-paying block.

Ordinals depend only on how many satoshis *could* have been mined in previous
blocks, not how many were *actually* mined.

In particular, this means that block 124724, which underpaid the subsidy by one
satoshi, does not reduce the ordinal ranges of subsequent blocks.

The `range` command gives the half-open range of ordinals that could be mined
in the block at a given height:

```
$ ord range 0
[0,5000000000)
```

## Transfer

The ordinal numbers on satoshis input to a transaction are transferred to the
transaction outputs in first-in-first-out order.

Satoshis paid as fees are considered to be inputs to the coinbase transaction,
after an implicit input containing the block subsidy, in the same order that
their parent transactions appear in the block.

If the coinbase transaction underpays the block subsidy or fees, those
satoshis, along with their ordinal numbers, are permanently destroyed.

The `find` command gives the satpoint containing the satoshi with a given
ordinal:

```
$ ord find 0
4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0
```

A satpoint is an outpoint, that is to say a transaction ID and output index,
followed by the offset into the output itself, and gives the position of the
satoshi within a particular output.

## Traits

Satoshis have traits, based on their ordinal.

NB: Traits should be considered *UNSTABLE* and subject to change.

The `traits` command prints out the traits of the satoshi with the given
ordinal:

```
$ ord traits 0
even
square
cube
luck: 0/1
population: 0
name: nvtdijuwxlo
character: '\u{0}'
shiny
block: 0
```

## Names

Each satoshi is assigned a name, consisting of lowercase ASCII characters,
based on its ordinal. Satoshi 0 has name `nvtdijuwxlo`, and names get shorter
as the ordinal number gets larger. This is to ensure that short names aren't
locked in the genesis block output which is unspendable, and other outputs,
which are unlikely to ever be spent.

The `name` command prints the ordinal of the satoshi with the given name:

```
$ ord name nvtdijuwxlo
0
$ ord name hello
2099999993937872
$ ord name ''
2099999997689999
```
