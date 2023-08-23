Testing
================

Ord can be tested using the following flags to specify the test network. For more information on runing bitcoin core for testing, see [Bitcoins developer documentation](https://developer.bitcoin.org/examples/testing.html).

Most ord commands in [inscriptions](inscriptions.md) and [explorer](explorer.md) can be run with the following network flags:


| Network | Flag |
|---------|------|
| Testnet | `--testnet` or `-t` |
| Signet  | `--signet` or `-s` |
| Regtest | `--regtest` or `-r` |

Regtest doesnt require downloading the blockchain or indexing ord.
### Example ord regtest workflow
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
Mine 101 blocks with: 
```
bitcoin-cli generatetoaddress 101 <receive address>
```
Inscribe in regtest with:
```
ord -r wallet inscribe --fee-rate 1 <file>
```
Mine the inscription with:
```
bitcoin-cli generatetoaddress 1 <receive address>
```
View the inscription in the regtest explorer:
```
ord -r server
```

Testing Recursion
---------
When testing out [recursion](../inscriptions/recursion.md), inscribe those dependencies first. The blocks don't need to be mined yet. The new regtest `inscriptionId` can then be used to test the recursion. Multiple blocks might need to be mined depending on how many files are being inscribed.