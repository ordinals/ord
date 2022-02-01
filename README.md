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

## Index and Caveats

The `ord` command builds an index using the contents of a local `bitcoind`'s
data directory, which must be halted while the index is built. Currently, the
index is built every time the `ord` runs, but that is a temporary limitation.
Reorgs are also not properly handled.

The index is stored in `index.redb`, and should not be concurrently modified
while an instance of `ord` is running, or used by two `ord` instances
simultaneously.

## Numbering

Satoshis are assigned ordinal numbers in the order in which they are mined.
Ordinals start at 0, for the first satoshi of the genesis block, and end with
2099999997689999, the only satoshi mined in block 6929999, the last block with
a subsidy.

Ordinals depend only on how many satoshis *could* have been mined in previous
blocks, not how many were *actually* mined.

In particular, this means that block 124724, which underpaid the block subsidy
by one satoshi, does not reduce the ordinals of satoshis in subsequent blocks.

The `range` command gives the half-open range of ordinals mined in the block at
a given height:

```
$ ord range 0
[0,5000000000)
```

See [src/range.rs](src/range.rs) for the numbering algorithm.

## Transfer

The ordinal numbers on satoshis input to a transaction are transferred to the
transaction outputs in first-in-first-out order.

Satoshis paid as fees are considered to be inputs to the coinbase transaction,
after an implicit input containing the block subsidy, in the same order that
their parent transactions appear in the block.

```rust
fn transfer(transaction: Transaction) {
  let mut ordinals: Vec<u64> = Vec::new();

  for input in transaction.inputs {
    for ordinal in input.ordinals {
      ordinals.push(ordinal);
    }
  }

  for output in transaction.outputs {
    for ordinal in &ordinals[0..output.value] {
      output.ordinals.push(ordinal);
    }
    ordinals = ordinals.split_off(output.value);
  }

  for ordinal in ordinals {
    coinbase.input.ordinals.push(ordinals);
  }
}
```

If the coinbase transaction underpays the block subsidy or fees, those
satoshis, along with their ordinal numbers, are destroyed and taken out of
circulation.

The `find` command, as of yet unfinished, gives the current satpoint containing
the satoshi with a given ordinal at a given height:

```
$ ord find --blocksdir ~/.bicoin/blocks 0 0
4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0
```

A satpoint is an outpoint, that is to say a transaction ID and output index,
followed by the offset into the output itself, and gives the position of the
satoshi within a particular output.

## Traits

Satoshis have traits, based on their ordinal.

NB: Traits should be considered *UNSTABLE*. In particular, the satoshis with
short names will not be available for quite some time, which might be desirable
to fix, and would require an overhaul of the name trait.

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
