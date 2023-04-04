Rest API Documentation
======================

This API allows you to query the Ordinal block explorer as well as interact with 
your wallet to create and manage inscriptions.

Where full URLs are provided in responses they will be rendered as if service
is running on 'https://ordinals.com/'.

## Open Endpoints

Open endpoints require no Authentication.

* [Feed](#feed) : `GET /feed`
* [Inscription details](#inscription) : `GET /inscription/:inscription_id`
* [Sat details](#sat) : `GET /sat/:sat`
* [Transaction details](#transaction) : `GET /tx/:tx`
* [Output details](#output) : `GET /output/:tx`
* [Block details](#block) : `GET /block/:block_num`
* [Content](#content) : `GET /content/:inscription_id`
* [Preview (thumbnail)](#preview) : `GET /preview/:inscription_id`

## Endpoints that require Authentication

The wallet endpoint require a valid API key `x-api-key` to be included in the header of the
request. The API key is set with the `api_key` field in the `ord.yaml` located in the ordinal configuration directory.

### Wallet Endpoints

Endpoints for viewing and manipulating the Wallet that the Authenticated User
has permissions to access.

* [Get Wallet Balance](#balance) : `GET /wallet/balance`
* [Create Wallet Receiving Address](#receive) : `GET /wallet/receive`
* [See All Wallet Transactions](#receive) : `GET /wallet/transactions`
* [Inscribe Inscription](#inscribe) : `POST /wallet/inscribe`
* [Send Inscription](#send) : `POST /wallet/send`

# Feed

Get the latest Ordinal inscriptions (latest 300).

**URL** : `/feed`

**Method** : `GET`

**Auth required** : NO

**Data**: `{}`

## Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "_links": {
    "inscriptions": [
      {
        "href": "/inscription/c2656bb3907f6dc21c9297f631108fb165933d58b5f633dee619594e27c0d414i0",
        "title": "Inscription 213970"
      },
      {
        "href": "/inscription/9dc46c5494cc663b07294c3bfbfb10ee69204252e9c80de9b33d1245550d8e11i0",
        "title": "Inscription 213969"
      },
      ... Results truncated
    ]
  }
}
```

# Inscription

Get the inscription details.

**URL** : `/inscription/:inscription_id`

**URL Parameters** : `inscription_id=[string]` where `inscription_id` is the ID of the Inscription on the
explorer.

**Method** : `GET`

**Auth required** : NO

**Data**: `{}`

## Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "_links": {
    "content": {
      "href": "/content/ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039i0"
    }, 
    "genesis_transaction": {
      "href": "/tx/ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039"
    }, 
    "next": {
      "href": "/inscription/632fac2718cb902eb2172d4b66e75e508d4f5ba9f2290df309959c1a5b8cc3fdi0"
    }, 
    "output": {
      "href": "/output/ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039:0"
    }, 
    "prev": {
      "href": "/inscription/5c34b2bc3ac76c4a4ce9fde4b6a6d7795d7835dfe6305c4cb4bb0cd026d63a5ci0"
    }, 
    "preview": {
      "href": "/preview/ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039i0"
    }, 
    "sat": null, 
    "self": {
      "href": "/inscription/ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039i0"
    }
  }, 
  "address": "ltc1q2f23v4kn5fcwjeayvfuj3e2q46cqmqdtzd4mec", 
  "content_length": 21039, 
  "content_type": "image/png", 
  "genesis_fee": 5818, 
  "genesis_height": 2442205, 
  "genesis_transaction": "ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039", 
  "location": "ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039:0:0", 
  "number": 213966, 
  "offset": 0, 
  "output": "ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039:0", 
  "sat": null, 
  "timestamp": "2023-03-20 20:08:40 UTC"
}
```

# Sat

Get sat details.

**URL** : `/sat/:sat`

**URL Parameters** : `sat=[integer]` where `sat` is the sat number on the explorer.

**Method** : `GET`

**Auth required** : NO

**Data**: `{}`

## Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "_links": {
    "block": {
      "href": "/block/0"
    }, 
    "inscription": null, 
    "next": {
      "href": "/sat/10000001"
    }, 
    "prev": {
      "href": "/sat/9999999"
    }, 
    "self": {
      "href": "/sat/10000000"
    }
  }, 
  "block": 0, 
  "cycle": 0, 
  "decimal": "0.10000000", 
  "degree": "0°0′0″10000000‴", 
  "epoch": 0, 
  "name": "bgmbqkpmtuab", 
  "offset": 10000000, 
  "percentile": "0.00000011904761917857144%", 
  "period": 0, 
  "rarity": "common", 
  "timestamp": "2011-10-07 07:31:05 UTC"
}
```

# Transaction

Get transaction.

**URL** : `/tx/:tx`

**URL Parameters** : `tx=[string]` where `tx` is the transaction on the blockchain.

**Method** : `GET`

**Auth required** : NO

**Data**: `{}`

## Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "_links": {
    "block": null, 
    "inputs": [
      {
        "href": "/output/4d4971a90f19470d244cb91e0ff2263af548b624fc1d102c983f7b29dbad8049:0"
      }
    ], 
    "inscription": {
      "href": "/inscription/ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039i0"
    }, 
    "outputs": [
      {
        "href": "/output/ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039:0"
      }
    ], 
    "self": {
      "href": "/tx/ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039"
    }
  }, 
  "transaction": "ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039"
}
```

# Output

Get transaction outputs.

**URL** : `/output/:tx`

**URL Parameters** : `tx=[string]` where `tx` is the transaction on the blockchain.

**Method** : `GET`

**Auth required** : NO

**Data**: `{}`

## Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "_links": {
    "self": {
      "href": "/output/ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039:0"
    }, 
    "transaction": {
      "href": "/tx/ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039"
    }
  }, 
  "address": "ltc1q2f23v4kn5fcwjeayvfuj3e2q46cqmqdtzd4mec", 
  "script_pubkey": "OP_0 OP_PUSHBYTES_20 52551656d3a270e967a4627928e540aeb00d81ab", 
  "transaction": "ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039", 
  "value": 10000
}
```

# Block

Get block.

**URL** : `/block/:block_num`

**URL Parameters** : `block_num=[integer]` where `block_num` is the block number on the blockchain.

**Method** : `GET`

**Auth required** : NO

**Data**: `{}`

## Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "_links": {
    "prev": {
      "href": "/block/12a765e31ffd4059bada1e25190f6e98c99d9714d334efa41a195a7e7e04bfe2"
    }, 
    "self": {
      "href": "/block/80ca095ed10b02e53d769eb6eaf92cd04e9e0759e5be4a8477b42911ba49c78f"
    }
  }, 
  "hash": "80ca095ed10b02e53d769eb6eaf92cd04e9e0759e5be4a8477b42911ba49c78f", 
  "previous_blockhash": "12a765e31ffd4059bada1e25190f6e98c99d9714d334efa41a195a7e7e04bfe2", 
  "size": 215, 
  "target": "00000ffff0000000000000000000000000000000000000000000000000000000", 
  "timestamp": "2011-10-08 06:29:19 UTC", 
  "weight": 860
}
```

# Content

Get inscription content.

**URL** : `/content/:inscription_id`

**URL Parameters** : `inscription_id=[string]` where `inscription_id` is the inscription id on the explorer.

**Method** : `GET`

**Auth required** : NO

**Data**: `{}`

## Success Response

**Code** : `200 OK`

**Content example**

`Response as file content.`

# Preview

Get inscription thumbnail preview.

**URL** : `/preview/:inscription_id`

**URL Parameters** : `inscription_id=[string]` where `inscription_id` is the inscription id on the explorer.

**Method** : `GET`

**Auth required** : NO

**Data**: `{}`

## Success Response

**Code** : `200 OK`

**Content example**

`Response as thumbnail image preview of the content.`

# Balance

Get wallet balance.

**URL** : `/wallet/balance`

**Method** : `GET`

**Auth required** : YES

**Data** : `{}`

## Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "_links":
  {
    "self":
    {
      "href": "/wallet/balance"
    }
  },
  "cardinal":29776159
}
```

# Receive

Generate a new receiving wallet address.

**URL** : `/wallet/receive`

**Method** : `GET`

**Auth required** : YES

**Data** : `{}`

## Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "_links":
  {
    "self":
    {
      "href": "/wallet/receive"
    }
  },
  "address": "tltc1qxymtfz3kmcu72twmcwskq2dejhnnszsxj272s4"
}
```

# Transactions

See all wallet transactions.

**URL** : `/wallet/transactions`

**Method** : `GET`

**Auth required** : YES

**Data** : `{}`

## Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "_links": {
    "self": {
      "href": "/wallet/transactions"
    }
  },
  "count": 979,
  "transactions": [
    {
      "_links": {
        "transaction": {
          "href": "/tx/92ba6106b74e6973af91dcc184966ba2751f26ce375982c455aa12cbcfc583c9"
        }
      },
      "confirmations": 69799,
      "transaction": "92ba6106b74e6973af91dcc184966ba2751f26ce375982c455aa12cbcfc583c9"
    },
    {
      "_links": {
        "transaction": {
          "href": "/tx/9586f83204057e61633dcf98b1d609dfdb8c86770bb01e60e1d86f7b0f25600e"
        }
      },
      "confirmations": 69463,
      "transaction": "9586f83204057e61633dcf98b1d609dfdb8c86770bb01e60e1d86f7b0f25600e"
    }
  ]
}
```

# Inscribe

Inscribe an ordinal from your wallet.

**URL** : `/wallet/inscribe`

**Method** : `POST`

**Auth required** : YES

**Data constraints**

Provide details for the ordinal to be sent.

```json
{
  "destination": "[destination address]",
  "fee_rate": "[decimal]",
  "no_backup": [boolean],
  "no_limit": [boolean],
  "dry_run": [boolean],
  "file": "[file path]"
}
```

**Data example** All fields must be sent.

```json
{
  "destination": "tltc1qxymtfz3kmcu72twmcwskq2dejhnnszsxj272s4",
  "fee_rate": "1.1",
  "no_backup": false,
  "no_limit": false,
  "dry_run": false, 
  "file": "C:\\ord\\upload\\file.png"
}
```

## Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "_links": 
  {
    "self":
    {
      "href": "/wallet/inscribe"
    }
  },
  "commit_tx": "cbc2eca5a0af2751bcb198dee36bbca969853e9304bed7dde593ac9f5ee5468c",
  "reveal_tx": "29e8b65f0f0122c26c018a7e3e6ffeabb6e357f2f0f05bb97868f4b5ca158d6c",
  "inscription_id": "29e8b65f0f0122c26c018a7e3e6ffeabb6e357f2f0f05bb97868f4b5ca158d6ci0",
  "fees": 359
}
```

# Send

Send an ordinal from your wallet.

**URL** : `/wallet/send`

**Method** : `POST`

**Auth required** : YES

**Data constraints**

Provide details for the ordinal to be sent.

```json
{
  "fee_rate": "[decimal]",
  "address": "[destination address]",
  "outgoing": "[inscription id, satpoint, amount]"
}
```

**Data example** All fields must be sent.

```json
{
  "fee_rate": "1.3",
  "address": "ltc1q2f23v4kn5fcwjeayvfuj3e2q46cqmqdtzd4mec",
  "outgoing": "ae1266bdb59ba2794471100beea8ce6449d901e9ceead477f3e1fa61d841a039i0"
}
```

## Success Response

**Code** : `200 OK`

**Content example**

```json
{
  "_links":
  {
    "self":
    {
      "href": "/wallet/send"
    }
  },
  "txid": "6904a154ceba5016e1b2b94f4c885bfc5c7a83c394e5fc1bd9553b934fd1d9c7"
}
```