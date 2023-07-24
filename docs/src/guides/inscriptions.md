Ordinal Inscription Guide
=========================

Individual sats can be inscribed with arbitrary content, creating
Bitcoin-native digital artifacts that can be held in a Bitcoin wallet and
transferred using Bitcoin transactions. Inscriptions are as durable, immutable,
secure, and decentralized as Bitcoin itself.

Working with inscriptions requires a Bitcoin full node, to give you a view of
the current state of the Bitcoin blockchain, and a wallet that can create
inscriptions and perform sat control when constructing transactions to send
inscriptions to another wallet.

Bitcoin Core provides both a Bitcoin full node and wallet. However, the Bitcoin
Core wallet cannot create inscriptions and does not perform sat control.

This requires [`ord`](https://github.com/ordinals/ord), the ordinal utility. `ord`
doesn't implement its own wallet, so `ord wallet` subcommands interact with
Bitcoin Core wallets.

This guide covers:

1. Installing Bitcoin Core
2. Syncing the Bitcoin blockchain
3. Creating a Bitcoin Core wallet
4. Using `ord wallet receive` to receive sats
5. Creating inscriptions with `ord wallet inscribe`
6. Sending inscriptions with `ord wallet send`
7. Receiving inscriptions with `ord wallet receive`

Getting Help
------------

If you get stuck, try asking for help on the [Ordinals Discord
Server](https://discord.com/invite/87cjuz4FYg), or checking GitHub for relevant
[issues](https://github.com/ordinals/ord/issues) and
[discussions](https://github.com/ordinals/ord/discussions).

Installing Bitcoin Core
-----------------------

Bitcoin Core is available from [bitcoincore.org](https://bitcoincore.org/) on
the [download page](https://bitcoincore.org/en/download/).

Making inscriptions requires Bitcoin Core 24 or newer.

This guide does not cover installing Bitcoin Core in detail. Once Bitcoin Core
is installed, you should be able to run `bitcoind -version` successfully from
the command line.

Configuring Bitcoin Core
------------------------

`ord` requires Bitcoin Core's transaction index.

To configure your Bitcoin Core node to maintain a transaction
index, add the following to your `bitcoin.conf`:

```
txindex=1
```

Or, run `bitcoind` with `-txindex`:

```
bitcoind -txindex
```

Syncing the Bitcoin Blockchain
------------------------------

To sync the chain, run:

```
bitcoind -txindex
```

…and leave it running until `getblockcount`:

```
bitcoin-cli getblockcount
```

agrees with the block count on a block explorer like [the mempool.space block
explorer](https://mempool.space/). `ord` interacts with `bitcoind`, so you
should leave `bitcoind` running in the background when you're using `ord`.

Installing `ord`
----------------

The `ord` utility is written in Rust and can be built from
[source](https://github.com/ordinals/ord). Pre-built binaries are available on the
[releases page](https://github.com/ordinals/ord/releases).

You can install the latest pre-built binary from the command line with:

```sh
curl --proto '=https' --tlsv1.2 -fsLS https://ordinals.com/install.sh | bash -s
```

Once `ord` is installed, you should be able to run:

```
ord --version
```

Which prints out `ord`'s version number.

Creating a Bitcoin Core Wallet
------------------------------

`ord` uses Bitcoin Core to manage private keys, sign transactions, and
broadcast transactions to the Bitcoin network.

To create a Bitcoin Core wallet named `ord` for use with `ord`, run:

```
ord wallet create
```

Receiving Sats
--------------

Inscriptions are made on individual sats, using normal Bitcoin transactions
that pay fees in sats, so your wallet will need some sats.

Get a new address from your `ord` wallet by running:

```
ord wallet receive
```

And send it some funds.

You can see pending transactions with:

```
ord wallet transactions
```

Once the transaction confirms, you should be able to see the transactions
outputs with `ord wallet outputs`.

Creating Inscription Content
----------------------------

Sats can be inscribed with any kind of content, but the `ord` wallet only
supports content types that can be displayed by the `ord` block explorer.

Additionally, inscriptions are included in transactions, so the larger the
content, the higher the fee that the inscription transaction must pay.

Inscription content is included in transaction witnesses, which receive the
witness discount. To calculate the approximate fee that an inscribe transaction
will pay, divide the content size by four and multiply by the fee rate.

Inscription transactions must be less than 400,000 weight units, or they will
not be relayed by Bitcoin Core. One byte of inscription content costs one
weight unit. Since an inscription transaction includes not just the inscription
content, limit inscription content to less than 400,000 weight units. 390,000
weight units should be safe.

Creating Inscriptions
---------------------

To create an inscription with the contents of `FILE`, run:

```
ord wallet inscribe --fee-rate FEE_RATE FILE
```

Ord will output two transactions IDs, one for the commit transaction, and one
for the reveal transaction, and the inscription ID. Inscription IDs are of the
form `TXIDiN`, where `TXID` is the transaction ID of the reveal transaction,
and `N` is the index of the inscription in the reveal transaction.

The commit transaction commits to a tapscript containing the content of the
inscription, and the reveal transaction spends from that tapscript, revealing
the content on chain and inscribing it on the first sat of the input that
contains the corresponding tapscript.

Wait for the reveal transaction to be mined. You can check the status of the
commit and reveal transactions using  [the mempool.space block
explorer](https://mempool.space/).

Once the reveal transaction has been mined, the inscription ID should be
printed when you run:

```
ord wallet inscriptions
```

And when you visit [the ordinals explorer](https://ordinals.com/) at
`ordinals.com/inscription/INSCRIPTION_ID`.

Sending Inscriptions
--------------------

Ask the recipient to generate a new address by running:

```
ord wallet receive
```

Send the inscription by running:

```
ord wallet send --fee-rate <FEE_RATE> <ADDRESS> <INSCRIPTION_ID>
```

See the pending transaction with:

```
ord wallet transactions
```

Once the send transaction confirms, the recipient can confirm receipt by
running:

```
ord wallet inscriptions
```

Receiving Inscriptions
----------------------

Generate a new receive address using:

```
ord wallet receive
```

The sender can transfer the inscription to your address using:

```
ord wallet send ADDRESS INSCRIPTION_ID
```

See the pending transaction with:
```
ord wallet transactions
```

Once the send transaction confirms, you can can confirm receipt by running:

```
ord wallet inscriptions
```
