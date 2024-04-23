use crate::{Chain, IndexExtension, RuneEntry};
use anyhow::Result;
use bitcoin::{
  block::{Header, Version},
  consensus::{encode::Error, Decodable},
  hash_types::TxMerkleNode,
  BlockHash, CompactTarget, Transaction, Txid,
};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use hex::FromHex;
use hyper::body::Buf;
use ordinals::{Artifact, Etching, RuneId, Runestone, SpacedRune};
use serde_json::json;
use std::{env, str::FromStr};
#[test]
fn test_rune_entry_index() {
  let txid =
    Txid::from_str("c11ee77ecfebc411a6cc171cc7aaafe00f04812264ee62fb040b18decb364279").unwrap();
  let block_hash =
    BlockHash::from_str("00000000002f23bfc94f42cc06031b960f7da0ec9ba96c2f744cc5615712ccac").ok();
  let (Ok(rpc_url), Ok(rpc_username), Ok(rpc_password)) = (
    env::var("ORD_BITCOIN_RPC_URL"),
    env::var("ORD_BITCOIN_RPC_USERNAME"),
    env::var("ORD_BITCOIN_RPC_PASSWORD"),
  ) else {
    panic!("Cannot connect to the BITCOIN node");
  };
  let rpc = Client::new(rpc_url.as_str(), Auth::UserPass(rpc_username, rpc_password)).unwrap();

  let raw_transaction = rpc.get_raw_transaction_hex(&txid, block_hash.as_ref());
  assert!(raw_transaction.is_ok());
  let raw_transaction = raw_transaction.unwrap();
  println!("{:?}", raw_transaction);
  let transaction = parse_transaction(raw_transaction.as_str());
  assert!(transaction.is_ok());
  let transaction = transaction.unwrap();
  let artifact = Runestone::decipher(&transaction);
  let extension = IndexExtension::new(Chain::from_str("testnet").unwrap());
  let vec_out =
    extension.index_transaction_output(1, 0, &txid, &transaction.output, artifact.as_ref());
  println!("{:?}", &vec_out);
  assert_eq!(vec_out.len(), 2);
  let txout = vec_out.get(1).unwrap();
  // assert!(txout.runestone.etching.is_some());
  // assert_eq!(txout.etching, txout.runestone.etching.is_some());
}

#[test]
fn test_mintentry_etching() {
  let id = RuneId {
    block: 2585710,
    tx: 1,
  };
  let block_time = 1712636172;
  let txid =
    Txid::from_str("bc0ee205a8e07d785fab5595099e9adb542f006443a25a2da3c42e465f10c34e").unwrap();
  let payload: &str = "0100000000010133ed4e69f8491c242cba5cca6c5f1e738724783298ce412b6081446d84cf16c20000000000fdffffff020000000000000000276a5d24020104c5d2e0cce19896adc18fb6b6b906010a03909104055406904e1601000000904e0101000000000000002251208c0b89163787ce8e5ce3f1262745024c91a6e488afb89f83a6ccaaa4db64b135034018007c4090d37b322f4a04a16f959924379a6e0c6ac14e200e8dbe996bcc64262e9a9324223456b79b1e7c7c90140f7c790c027303c4d3a338cdb9454f3c9e843220f77af935250eff2a1f0f049c6c974aff498c38519833ea78f1ecc4575ece44d7ac00630c45299819c6585ac187cd96336821c1f77af935250eff2a1f0f049c6c974aff498c38519833ea78f1ecc4575ece44d700000000";
  let transaction = parse_transaction(payload);
  assert!(transaction.is_ok());
  let transaction = transaction.unwrap();
  let artifact = Runestone::decipher(&transaction);
  assert!(artifact.is_some());
  let artifact = artifact.unwrap();
  let rune = match &artifact {
    Artifact::Runestone(runestone) => match runestone.etching {
      Some(etching) => etching.rune.clone(),
      None => None,
    },
    Artifact::Cenotaph(cenotaph) => match cenotaph.etching {
      Some(rune) => Some(rune.clone()),
      None => None,
    },
  };
  if let Some(rune) = rune {
    let number = 1;
    let entry = match artifact {
      Artifact::Cenotaph(_) => RuneEntry {
        block: id.block,
        burned: 0,
        divisibility: 0,
        etching: txid,
        terms: None,
        mints: 0,
        number,
        premine: 0,
        spaced_rune: SpacedRune { rune, spacers: 0 },
        symbol: None,
        timestamp: block_time,
        turbo: false,
      },
      Artifact::Runestone(Runestone { etching, .. }) => {
        let Etching {
          divisibility,
          terms,
          premine,
          spacers,
          symbol,
          turbo,
          ..
        } = etching.unwrap();

        RuneEntry {
          block: id.block,
          burned: 0,
          divisibility: divisibility.unwrap_or_default(),
          etching: txid,
          terms,
          mints: 0,
          number,
          premine: premine.unwrap_or_default(),
          spaced_rune: SpacedRune {
            rune,
            spacers: spacers.unwrap_or_default(),
          },
          symbol,
          timestamp: block_time,
          turbo,
        }
      }
    };
    //assert!(entry.terms.is_some());
  };
}

#[test]
fn test_index_transaction_with_etching() {
  let _block_hash = "0000000000000000c15b4efbffa470c066377d09087a030e4b1f5e4ef5aca838";
  let txid =
    Txid::from_str("7c946a72314d41ac080b2d4b1e1e2ad347c655cc3b5ba07b6f3f7ad3d26d6db0").unwrap();
  let payload: &str = "010000000001019681b514daa81c45bb549c01fc71ac6b77bdd276573aa09b3fe66d570fa276790000000000fdffffff0200000000000000001a6a5d17020104ccad90e1d8e0e116010103a2020680c2d72f160101000000000000002251208c0b89163787ce8e5ce3f1262745024c91a6e488afb89f83a6ccaaa4db64b13503401153898cc07296fb111e87de8638b76f91bc415fac5d7c09dfb4e81d7c334632890095dc2daa1c691acdc1567cbe47e0ecf22ac67aae60c0f7463d989ba7c6a32d20fb8d393fc7cc9bf972b99ef827555c126b8e333cf29241da3c5ae1e4cab86e86ac006307cc16248c05872d6821c0fb8d393fc7cc9bf972b99ef827555c126b8e333cf29241da3c5ae1e4cab86e8600000000";
  let transaction = parse_transaction(payload);
  assert!(transaction.is_ok());
  let transaction = transaction.unwrap();
  let artifact = Runestone::decipher(&transaction);
  let header = Header {
    version: Version::TWO,
    prev_blockhash: BlockHash::from_str(
      "00000000001bce60a31d34ab34924bc337d934fd00c24bc19eec3443ffbd6866",
    )
    .unwrap(),
    merkle_root: TxMerkleNode::from_str(
      "19678f485e698e57cce27d1c83d3db637f69425c56de1e53420c7f0837041174",
    )
    .unwrap(),
    time: 0,
    bits: CompactTarget::from_consensus(421978704),
    nonce: 1844512382,
  };
  let extension = IndexExtension::new(Chain::from_str("testnet").unwrap());
  let vec_out =
    extension.index_transaction_output(1, 0, &txid, &transaction.output, artifact.as_ref());
  println!("{:?}", &vec_out);
  assert_eq!(vec_out.len(), 2);
  let txout = vec_out.get(0).unwrap();
  println!("{:?}", txout);
  // assert_eq!(txout.etching, txout.runestone.etching.is_some());
}

#[test]
fn test_index_transaction_with_edicts() {
  let _block_hash = "00000000b622ddc1983ef0ee643801699cf5676f532592e68d1c8c0bcab0e903";
  let txid =
    Txid::from_str("2919534fee5ef7325059871e96876e4c8c16238da009d41734dd6e4d89d63af0").unwrap();
  let payload: &str = "020000000001021a6bd60dd2724f7e3cccf8696ddc90b9740afc6c62f5f19a4f3d16c39e5118890100000000ffffffffac5a9b44da02e1d578dffb57c66bdefa25b195617e7f0dced01fcf524414cf8f0000000000ffffffff05e9030000000000002251208c0b89163787ce8e5ce3f1262745024c91a6e488afb89f83a6ccaaa4db64b13500000000000000000e6a5d0b160000f0e29d011ee8070222020000000000002251208dc1576a4cf34a331d91d85d447173c97c30b4ef2ee4baab9ead6237dd4b09d8e803000000000000225120d02948e3c11f9035c2e225c325722d9701d1020c0ed7f8fe5320c17c56eaed69f4040000000000002251208dc1576a4cf34a331d91d85d447173c97c30b4ef2ee4baab9ead6237dd4b09d8014120378dea83e9695306c2e61e95eaccbb59183124d43621194fd9e47e0954fb4f91183b3409d27a56d048e4bdd15d30bd263b4ab8079d925a7ae17baac99142448301402eba2421f0958674c1b4de373da6e617dd7ea6e66d25fd4dab239f44461972f4c7b41fe98037305d461f2a5756ac45b7c5a6a878d10b516c1d757ff1f4bd360f00000000";
  let expect_runestone =
    json!({"edicts":[{"id":"2584944:30","amount":1000,"output":2}],"pointer":0});
  let transaction = parse_transaction(payload);
  assert!(transaction.is_ok());
  let transaction = transaction.unwrap();
  let artifact = Runestone::decipher(&transaction);
  let header = Header {
    version: Version::TWO,
    prev_blockhash: BlockHash::from_str(
      "00000000cd14fadcd151bca283c27a92738ad9376bdf8a8531e29226af6d7f9e",
    )
    .unwrap(),
    merkle_root: TxMerkleNode::from_str(
      "b54b4648d1f3ae159edae8bffd48d7db41890f9248e147daebffb18b51603bf9",
    )
    .unwrap(),
    time: 0,
    bits: CompactTarget::from_consensus(486604799),
    nonce: 1972576522,
  };
  let extension = IndexExtension::new(Chain::from_str("testnet").unwrap());
  let vec_out =
    extension.index_transaction_output(1, 0, &txid, &transaction.output, artifact.as_ref());
  assert_eq!(vec_out.len(), 5);
  let txout = vec_out.get(1).unwrap();
  let runestone = serde_json::to_string(&txout.runestone).unwrap_or_default();
  assert_eq!(runestone, serde_json::to_string(&expect_runestone).unwrap());
}

fn parse_transaction(payload: &str) -> Result<Transaction, Error> {
  let buffer = Vec::from_hex(payload).unwrap();
  let mut buffer_reader = buffer.reader();
  Transaction::consensus_decode_from_finite_reader(&mut buffer_reader)
}
