Ordinal Inscription Guide
=========================

Individual satoshis can be inscribed with arbitrary content, creating
Bitcoin-native digital artifacts that can be held in a Bitcoin wallet and
transferred using Bitcoin transactions. Inscriptions are as durable, immutable,
secure, and decentralized as Bitcoin itself.

Working with inscriptions requires a Bitcoin full node, to give you a view of
the current state of the Bitcoin blockchain, and a wallet that can make
ordinal-aware transactions that inscribe satoshis with content and transfer
individual satoshis using ordinal theory.

Bitcoin Core provides both a Bitcoin full node and wallet. However, the Bitcoin
Core wallet cannot make ordinal-aware transactions. Making ordinal-aware
transactions requires [`ord`](https://github.com/casey/ord), the ordinal theory
utility. `ord` doesn't implement its own wallet, so `ord wallet` subcommands
interact with an existing Bitcoin Core wallet.

This guide covers:

1. Installing Bitcoin Core
2. Syncing the Bitcoin blockchain
3. Creating a Bitcoin Core wallet
4. Using `ord wallet receive` to receive satoshis
5. Creating inscriptions with `ord wallet inscribe`
6. Sending inscriptions with `ord wallet send`
7. Receiving inscriptions with `ord wallet receive`

Getting Help
------------

If you get stuck, try asking for help on the [Ordinal Theory Discord
Server](https://discord.com/invite/87cjuz4FYg), or checking GitHub for relevant
[issues](https://github.com/casey/ord/issues) and
[discussions](https://github.com/casey/ord/discussions).

Installing Bitcoin Core
-----------------------

Bitcoin Core is available from [bitcoincore.org](https://bitcoincore.org/) on
the [download page](https://bitcoincore.org/en/download/).

This guide does not cover installing Bitcoin Core in detail. Once Bitcoin Core
is installed, you should be able to run `bitcoind -version` successfully from
the command line.

Configuring Bitcoin Core
------------------------

`ord wallet` subcommands cannot yet be used on mainnet, so your Bitcoin Core
node must be configured to use another chain. This guide uses signet, but
testnet or regtest mode may also be used. Additionally, `ord` requires Bitcoin
Core's transaction index.

To configure your Bitcoin Core node to use signet and maintain a transaction
index, add the following to your `bitcoin.conf`:

```
signet=1
txindex=1
```

Or, run `bitcoind` with `-signet` and `-txindex`:

```
bitcoind -signet -txindex
```

Syncing the Bitcoin Blockchain
------------------------------

Once Bitcoin Core has been configured to use signet, you'll need to sync the
blockchain. Signet is a low-volume test network, so this shouldn't take long.

To sync the chain, run:

```
bitcoind -signet -txindex
```

…and leave it running until `getblockchount`:

```
bitcoin-cli -signet getblockcount
```

agrees with the block count on a block explorer like [the mempool.space signet
block explorer](https://mempool.space/signet). `ord` interacts with `bitcoind`,
so you should leave `bitcoind` running in the background when you're using
`ord`.

Creating a Bitcoin Core Wallet
------------------------------

`ord` uses Bitcoin Core to manage private keys, sign transactions, and
broadcast transactions to the Bitcoin network.

`ord` wallets must be named `ord`, or start with `ord-`, to avoid
unintentionally using the `ord` utility with non-ordinal Bitcoin wallets.

To create a Bitcoin Core wallet named `ord` for use with `ord`, run:

```
ord --signet wallet create
```

This is equivalent to running:

```
bitcoin-cli -signet createwallet ord
```

Loading the Bitcoin Core Wallet
-------------------------------

Bitcoin Core wallets must be loaded before they can be used with `ord`. Wallets
are loaded automatically after being created, so you do not need to load your
wallet if you just created it. Otherwise, run:

```
bitcoin-cli -signet loadwallet ord
```

Once your wallet is loaded, you should be able to run:

```
ord --signet wallet receive
```

…and get a fresh receive address.

If you get an error message like:

```
error: JSON-RPC error: RPC error response: RpcError { code: -19, message: "Wallet file not specified (must request wallet RPC through /wallet/<filename> uri-path).", data: None }
```

This means that `bitcoind` has more than one wallet loaded. List loaded wallets with:

```
bitcoin-cli -signet listwallets
```

And unload all wallets other than `ord`:

```
bitcoin-cli -signet unloadwallet foo
```

Installing `ord`
----------------

The `ord` utility is written in Rust and can be built from
[source](https://github.com/casey/ord). Pre-built binaries are available on the
[releases page](https://github.com/casey/ord/releases).

You can install the latest pre-built binary from the command line with:

```sh
curl --proto '=https' --tlsv1.2 -fsLS https://ordinals.com/install.sh | bash -s
```

Once `ord` is installed, you should be able to run:

```
ord --version
```

Which prints out `ord`'s version number.

Receiving Satoshis
------------------

Inscriptions are made on individual satoshis, using normal Bitcoin transactions
that pay fees in satoshis, so your wallet will need some sats.

Get a new address from your `ord` wallet by running:

```
ord --signet wallet receive
```

Use a signet faucet to send satoshis to the address you generated. Two faucets
you might try are [signet.bc-2.jp](https://signet.bc-2.jp/) and
[alt.signetfaucet.com](https://alt.signetfaucet.com/).

You can see pending transactions with:

```
ord --signet wallet transactions
```

Once the faucet transaction confirms, you should be able to see the
transactions outputs with `ord --signet wallet utxos`.

Creating Inscription Content
----------------------------

Create a `.png` or `.txt` file smaller than 1024 bytes to inscribe.

Satoshis can be inscribed with any kind of content, but the `ord` wallet and
explorer are currently limited to `.png` and `.txt` files.

Additionally, inscriptions made on signet must be 1024 bytes or less, to avoid
congesting signet for other users. Inscriptions are stored in Taproot input
witnesses, so mainnet inscriptions will be limited only by the depths of your
pockets and the 4,000,000 byte witness size limit.

Creating Inscriptions
---------------------

To create an inscription with the contents of `FILE`, run:

```
ord --signet wallet inscribe --file FILE
```

Ord will output two transactions IDs, one for the commit transaction, and one
for the reveal transaction.

The commit transaction commits to a tapscript containing the contents of the
inscription, and the reveal transaction spends from that tapscript, revealing
the contents on chain and inscribing them on the first satoshi of the first
output of the reveal transaction.

Wait for the reveal transaction to be mined. You can check the status of the
commit and reveal transactions using  [the mempool.space signet block
explorer](https://mempool.space/signet).

Once the reveal transaction has been mined, the inscription ID should be
printed when you run:

```
ord --signet wallet inscriptions
```

And when you visit [the signet ordinals explorer](https://signet.ordinals.com/)
at `signet.ordinals.com/inscription/INSCRIPTION_ID`.


Sending Inscriptions
--------------------

Ask the recipient to generate a new address by running:

```
ord --signet wallet receive
```

Send the inscription by running:

```
ord --signet wallet send INSCRIPTION_ID ADDRESS
```

See the pending transaction with:
```
ord --signet wallet transactions
```

Once the send transaction confirms, the recipient can confirm receipt by
running:

```
ord --signet wallet inscriptions
```

Receiving Inscriptions
----------------------

Generate a new receive address using:

```
ord --signet wallet receive
```

The sender can transfer the inscription to your address using:

```
ord --signet wallet send INSCRIPTION_ID ADDRESS
```

See the pending transaction with:
```
ord --signet wallet transactions
```

Once the send transaction confirms, you can can confirm receipt by running:

```
ord --signet wallet inscriptions
```
