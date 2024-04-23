Recursion
=========

An important exception to [sandboxing](../inscriptions.md#sandboxing) is
recursion. Recursive endpoints are whitelisted endpoints that allow access to
on-chain data, including the content of other inscriptions.

Since changes to recursive endpoints might break inscriptions that rely on
them, recursive endpoints have backwards-compatibility guarantees not shared by
`ord server`'s other endpoints. In particular:

- Recursive endpoints will not be removed
- Object fields returned by recursive endpoints will not be renamed or change types

However, additional object fields may be added or reordered, so inscriptions
must handle additional, unexpected fields, and must not expect fields to be
returned in a specific order.

Recursion has a number of interesting use-cases:

- Remixing the content of existing inscriptions.

- Publishing snippets of code, images, audio, or stylesheets as shared public
  resources.

- Generative art collections where an algorithm is inscribed as JavaScript,
  and instantiated from multiple inscriptions with unique seeds.

- Generative profile picture collections where accessories and attributes are
  inscribed as individual images, or in a shared texture atlas, and then
  combined, collage-style, in unique combinations in multiple inscriptions.

The recursive endpoints are:

- `/content/<INSCRIPTION_ID>`:  the content of the inscription with `<INSCRIPTION_ID>`
- `/r/blockhash/<HEIGHT>`: block hash at given block height.
- `/r/blockhash`: latest block hash.
- `/r/blockheight`: latest block height.
- `/r/blockinfo/<QUERY>`: block info. `<QUERY>` may be a block height or block hash.
- `/r/blocktime`: UNIX time stamp of latest block.
- `/r/children/<INSCRIPTION_ID>`: the first 100 child inscription ids.
- `/r/children/<INSCRIPTION_ID>/<PAGE>`: the set of 100 child inscription ids on `<PAGE>`.
- `/r/inscription/<INSCRIPTION_ID>`: information about an inscription
- `/r/metadata/<INSCRIPTION_ID>`: JSON string containing the hex-encoded CBOR metadata.
- `/r/sat/<SAT_NUMBER>`: the first 100 inscription ids on a sat.
- `/r/sat/<SAT_NUMBER>/<PAGE>`: the set of 100 inscription ids on `<PAGE>`.
- `/r/sat/<SAT_NUMBER>/at/<INDEX>`: the inscription id at `<INDEX>` of all inscriptions on a sat. `<INDEX>` may be a negative number to index from the back. `0` being the first and `-1` being the most recent for example.

Note: `<SAT_NUMBER>` only allows the actual number of a sat no other sat
notations like degree, percentile or decimal. We may expand to allow those in
the future.

Responses from the above recursive endpoints are JSON. For backwards
compatibility additional endpoints are supported, some of which return
plain-text responses.

- `/blockheight`: latest block height.
- `/blockhash`: latest block hash.
- `/blockhash/<HEIGHT>`: block hash at given block height.
- `/blocktime`: UNIX time stamp of latest block.

Examples
--------

- `/r/blockhash/0`:

```json
"000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
```

- `/r/blockheight`:

```json
777000
```

- `/r/blockinfo/0`:

```json
{
  "average_fee": 0,
  "average_fee_rate": 0,
  "bits": 486604799,
  "chainwork": "0000000000000000000000000000000000000000000000000000000100010001",
  "confirmations": 0,
  "difficulty": 0.0,
  "hash": "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
  "height": 0,
  "max_fee": 0,
  "max_fee_rate": 0,
  "max_tx_size": 0,
  "median_fee": 0,
  "median_time": 1231006505,
  "merkle_root": "0000000000000000000000000000000000000000000000000000000000000000",
  "min_fee": 0,
  "min_fee_rate": 0,
  "next_block": null,
  "nonce": 0,
  "previous_block": null,
  "subsidy": 5000000000,
  "target": "00000000ffff0000000000000000000000000000000000000000000000000000",
  "timestamp": 1231006505,
  "total_fee": 0,
  "total_size": 0,
  "total_weight": 0,
  "transaction_count": 1,
  "version": 1
}
```

- `/r/blocktime`:

```json
1700770905
```

- `/r/children/60bcf821240064a9c55225c4f01711b0ebbcab39aa3fafeefe4299ab158536fai0/49`:

```json
{
   "ids":[
      "7cd66b8e3a63dcd2fada917119830286bca0637267709d6df1ca78d98a1b4487i4900",
      "7cd66b8e3a63dcd2fada917119830286bca0637267709d6df1ca78d98a1b4487i4901",
      ...
      "7cd66b8e3a63dcd2fada917119830286bca0637267709d6df1ca78d98a1b4487i4935",
      "7cd66b8e3a63dcd2fada917119830286bca0637267709d6df1ca78d98a1b4487i4936"
   ],
   "more":false,
   "page":49
}
```

- `/r/inscription/3bd72a7ef68776c9429961e43043ff65efa7fb2d8bb407386a9e3b19f149bc36i0`

```json
{
  "charms": [],
  "content_type": "image/png",
  "content_length": 144037,
  "fee": 36352,
  "height": 209,
  "id": "3bd72a7ef68776c9429961e43043ff65efa7fb2d8bb407386a9e3b19f149bc36i0",
  "number": 2,
  "output": "3bd72a7ef68776c9429961e43043ff65efa7fb2d8bb407386a9e3b19f149bc36:0",
  "sat": null,
  "satpoint": "3bd72a7ef68776c9429961e43043ff65efa7fb2d8bb407386a9e3b19f149bc36:0:0",
  "timestamp": 1708312562,
  "value": 10000
}
```

- `/r/metadata/35b66389b44535861c44b2b18ed602997ee11db9a30d384ae89630c9fc6f011fi3`:

```json
"a2657469746c65664d656d6f727966617574686f726e79656c6c6f775f6f72645f626f74"
```

- `/r/sat/1023795949035695`:

```json
{
   "ids":[
      "17541f6adf6eb160d52bc6eb0a3546c7c1d2adfe607b1a3cddc72cc0619526adi0"
   ],
   "more":false,
   "page":0
}
```

- `/r/sat/1023795949035695/at/-1`:

```json
{
   "id":"17541f6adf6eb160d52bc6eb0a3546c7c1d2adfe607b1a3cddc72cc0619526adi0"
}
```

- `/r/children/60bcf821240064a9c55225c4f01711b0ebbcab39aa3fafeefe4299ab158536fai0/49`:

```json
{
   "ids":[
      "7cd66b8e3a63dcd2fada917119830286bca0637267709d6df1ca78d98a1b4487i4900",
      "7cd66b8e3a63dcd2fada917119830286bca0637267709d6df1ca78d98a1b4487i4901",
      ...
      "7cd66b8e3a63dcd2fada917119830286bca0637267709d6df1ca78d98a1b4487i4935",
      "7cd66b8e3a63dcd2fada917119830286bca0637267709d6df1ca78d98a1b4487i4936"
   ],
   "more":false,
   "page":49
}
```
