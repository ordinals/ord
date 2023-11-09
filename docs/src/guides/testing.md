Testing
=======

Ord can be tested using the following flags to specify the test network. For more
information on running Bitcoin Core for testing, see [Bitcoin's developer documentation](https://developer.bitcoin.org/examples/testing.html).

Most `ord` commands in [inscriptions](inscriptions.md) and [explorer](explorer.md)
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
