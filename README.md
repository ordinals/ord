# Sat Tracker

A scheme for assigning serial numbers to satoshis upon creation and tracking
them across subsequent transactions.

Satoshi serial numbers can be used as an addressing scheme for NFTs.


## Numbering

Satoshis are numbered in the order in which they are mined.

Satoshi numbers only depend on how many satoshis could have been created in
previous blocks, not how many were *actually* created.

In particular, this means that block 124724, which underpaid the block subsidy
by one, does not reduce the serial numbers of satoshis in subsequent blocks by
one.

The `range` command gives the half-open range of satoshis mined in the block at
a given height:

```
$ sat-tracker range 0
0 50000000000
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

The `traits` command prints out the traits of a given satoshi:

```
$ sat-tracker traits 0
zero
genesis
even
```

## Names

The name trait is of particular interest, as it can serve as the basis for a
decentralized naming system.

The `name` command finds the satoshi with the given name:

```
$ sat-tracker name nvtdijuwxlp
2099999997690000
```
