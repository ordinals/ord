# Sat Tracker

A scheme for assigning serial numbers to satoshis upon creation and tracking
them across subsequent transactions.

Satoshi serial numbers can be used as an addressing scheme for NFTs.


## Numbering

Satoshis are numbered in ascending order, starting at 0 in the genesis block,
and ending with 2099999997689999, mined in block 6929999, the last block with a
subsidy.

Satoshi numbers only depend on how many satoshis could have been created in
previous blocks, not how many were *actually* created.

In particular, this means that block 124724, which underpaid the block subsidy
by one satoshi, does not reduce the serial numbers of satoshis in subsequent
blocks.

The `range` command gives the half-open range of satoshis mined in the block at
a given height:

```
$ sat-tracker range 0
[0,5000000000)
```

See [src/range.rs](src/range.rs) for the numbering algorithm.


## Transfer

Satoshis input to a transaction are transferred to the transaction outputs
according to the order and value of the inputs and outputs. Satoshis paid as
fees are assigned in the same fashion to the outputs of the coinbase
transaction.

```rust
fn transfer(transaction: Transaction) {
  let mut numbers = Vec::new();

  for input in transaction.inputs {
    for number in input.numbers {
      numbers.push(number);
    }
  }

  for output in transaction.outputs {
    let rest = numbers.split_off(output.value);
    output.numbers = numbers;
    numbers = rest;
  }

  coinbase.input.numbers.extend_from_slice(&numbers);
}
```

The `find` command, as of yet unfinished, gives the current outpoint containing
a given satoshi as of a given height:

```
$ sat-tracker find --blocksdir ~/.bicoin/blocks 0 0
4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0
```


## Traits

Satoshis have traits, based on their number.

NB: Traits should be considered *UNSTABLE*. In particular, the satoshis with
short names will not be available for quite some time, which might be desirable
to fix, and would require an overhaul of the name trait.

The `traits` command prints out the traits of a given satoshi:

```
$ sat-tracker traits 0
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

Each satoshi is assigned a name, consisting of lowercase ASCII characters.
Satoshi 0 has name `nvtdijuwxlo`, and names get shorter as the satoshi number
gets larger. This is to ensure that short names aren't locked in the genesis
block output which is unspendable, and other outputs, which are unlikely to
ever be spent.

The `name` command finds the satoshi with the given name:

```
$ sat-tracker name nvtdijuwxlo
0
$ sat-tracker name hello
2099999993937872
$ sat-tracker name ''
2099999997689999
```
