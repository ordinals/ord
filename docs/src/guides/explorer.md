Ordinal Explorer
================

The `ord` binary includes a block explorer. We host a instance of the block
explorer on mainnet at [ordinals.com](https://ordinals.com), and on signet at
[signet.ordinals.com](https://signet.ordinals.com).

### Running The Explorer
The server can be run locally with:

`ord server`

To specify a port add the `--http-port` flag:

`ord server --http-port 8080`

To enable the JSON-API endpoints add the `--enable-json-api` or `-j` flag (see
[here](#json-api) for more info):

`ord server --enable-json-api`

To test how your inscriptions will look you can run:

`ord preview <FILE1> <FILE2> ...`

Search
------

The search box accepts a variety of object representations.

### Blocks

Blocks can be searched by hash, for example, the genesis block:

[000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f](https://ordinals.com/search/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f)

### Transactions

Transactions can be searched by hash, for example, the genesis block coinbase
transaction:

[4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b](https://ordinals.com/search/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b)

### Outputs

Transaction outputs can searched by outpoint, for example, the only output of
the genesis block coinbase transaction:

[4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0](https://ordinals.com/search/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0)

### Sats

Sats can be searched by integer, their position within the entire bitcoin
supply:

[2099994106992659](https://ordinals.com/search/2099994106992659)

By decimal, their block and offset within that block:

[481824.0](https://ordinals.com/search/481824.0)

By degree, their cycle, blocks since the last halving, blocks since the last
difficulty adjustment, and offset within their block:

[1°0′0″0‴](https://ordinals.com/search/1°0′0″0‴)

By name, their base 26 representation using the letters "a" through "z":

[ahistorical](https://ordinals.com/search/ahistorical)

Or by percentile, the percentage of bitcoin's supply that has been or will have
been issued when they are mined:

[100%](https://ordinals.com/search/100%)

JSON-API
--------

You can run `ord server` with the `--enable-json-api` flag to access endpoints that
return JSON instead of HTML if you set the HTTP `Accept: application/json`
header. The structure of theses objects closely follows
what is shown in the HTML. These endpoints are:

- `/inscription/<INSCRIPTION_ID>`
- `/inscriptions`
- `/inscriptions/block/<BLOCK_HEIGHT>`
- `/inscriptions/block/<BLOCK_HEIGHT>/<PAGE_INDEX>`
- `/inscriptions/<FROM>`
- `/inscriptions/<FROM>/<N>`
- `/output/<OUTPOINT>`
- `/output/<OUTPOINT>`
- `/sat/<SAT>`

To get a list of the latest 100 inscriptions you would do:

```
curl -s -H "Accept: application/json" 'http://0.0.0.0:80/inscriptions'
```

To see information about a UTXO, which includes inscriptions inside it, do:

```
curl -s -H "Accept: application/json" 'http://0.0.0.0:80/output/bc4c30829a9564c0d58e6287195622b53ced54a25711d1b86be7cd3a70ef61ed:0'
```

Which returns:

```
{
  "value": 10000,
  "script_pubkey": "OP_PUSHNUM_1 OP_PUSHBYTES_32 156cc4878306157720607cdcb4b32afa4cc6853868458d7258b907112e5a434b",
  "address": "bc1pz4kvfpurqc2hwgrq0nwtfve2lfxvdpfcdpzc6ujchyr3ztj6gd9sfr6ayf",
  "transaction": "bc4c30829a9564c0d58e6287195622b53ced54a25711d1b86be7cd3a70ef61ed",
  "sat_ranges": null,
  "inscriptions": [
    "6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0"
  ]
}
```
