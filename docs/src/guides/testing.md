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
`bitcoin.conf`, and the env configuration to `env.json`.

`env.json` contains the commands needed to invoke `bitcoin-cli` and `ord
wallet`, as well as the ports `bitcoind` and `ord server` are listening on.

These can be extracted into shell commands using `jq`:

```shell
bitcoin=`jq -r '.bitcoin_cli_command | join(" ")' env/env.json`
$bitcoin listunspent

ord=`jq -r '.ord_wallet_command | join(" ")' env/env.json`
$ord outputs
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

Regtest doesn't require downloading the blockchain or indexing ord.

Example
-------

Run bitcoind in regtest with:
```
bitcoind -regtest -txindex
```

Create a wallet in regtest with:
```
ord -r wallet create
```

Get a regtest receive address with:
```
ord -r wallet receive
```

Mine 101 blocks (to unlock the coinbase) with:
```
bitcoin-cli -regtest generatetoaddress 101 <receive address>
```

Inscribe in regtest with:
```
ord -r wallet inscribe --fee-rate 1 --file <file>
```

Mine the inscription with:
```
bitcoin-cli -regtest generatetoaddress 1 <receive address>
```

View the inscription in the regtest explorer:
```
ord -r server
```

By default, browsers don't support compression over HTTP. To test compressed
content over HTTP, use the `--decompress` flag:
```
ord -r server --decompress
```

Testing Recursion
-----------------

When testing out [recursion](../inscriptions/recursion.md), inscribe the
dependencies first (example with [p5.js](https://p5js.org)):
```
ord -r wallet inscribe --fee-rate 1 --file p5.js
```
This should return a `inscription_id` which you can then reference in your
recursive inscription.

ATTENTION: These ids will be different when inscribing on
mainnet or signet, so be sure to change those in your recursive inscription for
each chain.

Then you can inscribe your recursive inscription with:
```
ord -r wallet inscribe --fee-rate 1 --file recursive-inscription.html
```
Finally you will have to mine some blocks and start the server:
```
bitcoin-cli generatetoaddress 6 <receive address>
ord -r server
```
