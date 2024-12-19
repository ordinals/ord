Testing
=======

Test Environment
----------------

`ord env <DIRECTORY>` creates a test environment in `<DIRECTORY>`, spins up
`bitcoind` and `ord server` instances, prints example commands for interacting
with the test `bitcoind` and `ord server` instances, waits for `CTRL-C`, and
then shuts down `bitcoind` and `ord server`.

`ord env` tries to use port 9000 for `bitcoind`'s RPC interface, and port
`9001` for `ord`'s RPC interface, but will fall back to random unused ports.

Inside of the env directory, `ord env` will write `bitcoind`'s configuration to
`bitcoin.conf`, `ord`'s configuration to `ord.yaml`, and the env configuration
to `env.json`.

`env.json` contains the commands needed to invoke `bitcoin-cli` and `ord
wallet`, as well as the ports `bitcoind` and `ord server` are listening on.

These can be extracted into shell commands using `jq`:

```shell
bitcoin=`jq -r '.bitcoin_cli_command | join(" ")' env/env.json`
$bitcoin listunspent

ord=`jq -r '.ord_wallet_command | join(" ")' env/env.json`
$ord outputs
```

If `ord` is in the `$PATH` and the env directory is `env`, the `bitcoin-cli`
command will be:

```
bitcoin-cli -datadir=env
```

And the `ord` will be:

```
ord --datadir env
```

Test Networks
-------------

Ord can be tested using the following flags to specify the test network. For more
information on running Bitcoin Core for testing, see [Bitcoin's developer documentation](https://developer.bitcoin.org/examples/testing.html).

Most `ord` commands in [wallet](wallet.md) and [explorer](explorer.md)
can be run with the following network flags:

| Network | Flag |
|---------|------|
| Testnet | `--testnet` or `-t` |
| Signet  | `--signet` or `-s` |
| Regtest | `--regtest` or `-r` |

Regtest doesn't require downloading the blockchain since you create your own
private blockchain, so indexing `ord` is almost instantaneous.

Example
-------

Run `bitcoind` in regtest with:

```
bitcoind -regtest -txindex
```

Run `ord server` in regtest with:

```
ord --regtest server
```

Create a wallet in regtest with:

```
ord --regtest wallet create
```

Get a regtest receive address with:

```
ord --regtest wallet receive
```

Mine 101 blocks (to unlock the coinbase) with:

```
bitcoin-cli -regtest generatetoaddress 101 <receive address>
```

Inscribe in regtest with:

```
ord --regtest wallet inscribe --fee-rate 1 --file <file>
```

Mine the inscription with:

```
bitcoin-cli -regtest generatetoaddress 1 <receive address>
```

By default, browsers don't support compression over HTTP. To test compressed
content over HTTP, use the `--decompress` flag:

```
ord --regtest server --decompress
```

Testing Recursion
-----------------

When testing out [recursion](../inscriptions/recursion.md), inscribe the
dependencies first (example with [p5.js](https://p5js.org)):

```
ord --regtest wallet inscribe --fee-rate 1 --file p5.js
```

This will return the inscription ID of the dependency which you can then
reference in your inscription.

However, inscription IDs differ between mainnet and test chains, so you must
change the inscription IDs in your inscription to the mainnet inscription IDs of
your dependencies before making the final inscription on mainnet.

Then you can inscribe your recursive inscription with:

```
ord --regtest wallet inscribe --fee-rate 1 --file recursive-inscription.html
```

Finally you will have to mine some blocks and start the server:

```
bitcoin-cli generatetoaddress 6 <receive address>
```

### Mainnet Dependencies

To avoid having to change dependency inscription IDs to mainnet inscription IDs,
you may utilize a content proxy when testing. `ord server` accepts a
`--proxy` option, which takes the URL of a another `ord server`
instance. When making a request to `/content/<INSCRIPTION_ID>` when a content
proxy is set and the inscription is not found, `ord server` will forward the
request to the content proxy. This allows you to run a test `ord server`
instance with a mainnet content proxy. You can then use mainnet inscription IDs
in your test inscription, which will then return the content of the mainnet
inscriptions.

```
ord --regtest server --proxy https://ordinals.com
```
