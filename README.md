`ord`
=====

Export inscriptions.

## Usage

### Example

```bash
./ord --index=~/btc/ord/index/index.redb --cookie-file=~/bitcoincore/.cookie index export --output ~/moe/btc/ord/dump/new_test_2 --gt-sequence 1 --lt-sequence 10000
job done. 9998 recorded(cursed: 13, p2pk: 0, unbound: 0, 0-body: 1) exported in 141.470550973s. 10001 inscriptions(<= 1 included, >= 10000 not included) in block heights: [0,843911)
Percentiles distribution of inscription body size(>0), min=1, max=3915775, mean=42466.51, stdev=81483.72:
|   1.00th=[4] (samples: 46)
|   5.00th=[221] (samples: 2)
|  10.00th=[727] (samples: 3)
|  20.00th=[1471] (samples: 1196)
|  30.00th=[2287] (samples: 28)
|  40.00th=[4415] (samples: 7)
|  50.00th=[11455] (samples: 5)
|  60.00th=[25983] (samples: 8)
|  70.00th=[41215] (samples: 11)
|  80.00th=[62719] (samples: 8)
|  90.00th=[111615] (samples: 9)
|  95.00th=[199679] (samples: 6)
|  99.00th=[370687] (samples: 8)
|  99.50th=[382975] (samples: 5)
|  99.90th=[393215] (samples: 3)
|  99.95th=[397311] (samples: 4)
|  99.99th=[3915775] (samples: 1)
```

## Pre-requisites

### Start `bitcond` server

```bash
./bitcoind -datadir=~/bitcoincore -txindex=1 -server=1
```

### Set `height` of `bitcoind`

#### Example

height 843911 block hash:

000000000000000000009fd14dd9da6a815083b2fb39d89619aeef583e094c72

set height to 843911:

```bash
./bitcoin-cli -datadir=~/bitcoincore -conf=~/bitcoincore/bitcoin.conf -rpccookiefile=~/bitcoincore/.cookie invalidateblock 000000000000000000009fd14dd9da6a815083b2fb39d89619aeef583e094c72
```

block range: [0, 843911)

## Output

```Rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InscriptionOutput {
  pub sequence_number: u32,
  pub inscription_number: i32,
  pub id: InscriptionId,
  // ord crate has different version of bitcoin dependency, using string for compatibility
  pub satpoint_outpoint: String, // txid:vout
  pub satpoint_offset: u64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub body: Option<Vec<u8>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content_encoding: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content_type: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<Vec<u8>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metaprotocol: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parent: Option<Vec<InscriptionId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pointer: Option<u64>,
  pub is_p2pk: bool, // If true, address field is script
  pub address: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rune: Option<u128>,
}```