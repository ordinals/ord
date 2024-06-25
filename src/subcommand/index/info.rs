use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Info {
  #[arg(long)]
  transactions: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionsOutput {
  pub start: u32,
  pub end: u32,
  pub count: u32,
  pub elapsed: f64,
}

impl Info {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    index.update()?;

    let info = index.info()?;

    if self.transactions {
      let mut output = Vec::new();
      for window in info.transactions.windows(2) {
        let start = &window[0];
        let end = &window[1];
        output.push(TransactionsOutput {
          start: start.starting_block_count,
          end: end.starting_block_count,
          count: end.starting_block_count - start.starting_block_count,
          elapsed: (end.starting_timestamp - start.starting_timestamp) as f64 / 1000.0 / 60.0,
        });
      }
      Ok(Some(Box::new(output)))
    } else {
      Ok(Some(Box::new(info)))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use bitcoin::consensus::deserzialize;
  use bitcoin::consensus::encode::serialize_hex;
  use bitcoin::secp256k1::PublicKey;
  use bitcoin::{Address, Script, ScriptBuf};

  // #[test]
  // fn test_gen_utxo_data_from_csv_line() {
  //     let script = "04c863781e34ee29d96f493a002de08c27cdfb79268d774b566100e9ae2d06bdfe4846faaa1fbe995fc969f64b503acc83203ea006f00b855355dbfb6f54b93211";
  //
  //     // let script2 = "4341049434a2dd7c5b82df88f578f8d7fd14e8d36513aaa9c003eb5bd6cb56065e44b7e0227139e8a8e68e7de0a4ed32b8c90edc9673b8a7ea541b52f2a22196f7b8cfac";
  //     let script2 = "210328e516b6a660d68009c57826bef9a5733b4b138dd125953175ee4664aca9fd0cac";
  //     let pubkey = PublicKey::from_str(script).unwrap();
  //     let pubkey_hex = serialize_hex(&pubkey.to_bytes());
  //     println!("pubkey: {:?}", serialize_hex(&pubkey.to_bytes()));
  //     println!("{}, {}, {}", pubkey_hex.len(), script2.len(), script.len());
  //
  //     let script2_buf = ScriptBuf::from_hex(&script2).unwrap();
  //     println!("{:?}", script2_buf.is_p2pk());
  //
  //     let pubkey2 = script2_buf.p2pk_public_key().unwrap();
  //     let pubkey2_hex = serialize_hex(&pubkey2.to_bytes());
  //     println!("pubkey2: {:?}", serialize_hex(&pubkey2.to_bytes()));
  //
  //     println!(
  //         "{}, {}",
  //         Address::from_script(&script2_buf, bitcoin::Network::Bitcoin).unwrap(),
  //         script2_buf.is_p2pk()
  //     );
  // }

  #[test]
  fn test_from_p2pk_script() {
    // let script = "04c863781e34ee29d96f493a002de08c27cdfb79268d774b566100e9ae2d06bdfe4846faaa1fbe995fc969f64b503acc83203ea006f00b855355dbfb6f54b93211";
    let script = "41049434a2dd7c5b82df88f578f8d7fd14e8d36513aaa9c003eb5bd6cb56065e44b7e0227139e8a8e68e7de0a4ed32b8c90edc9673b8a7ea541b52f2a22196f7b8cfac";
    let script = "210328e516b6a660d68009c57826bef9a5733b4b138dd125953175ee4664aca9fd0cac";
    // let script = &script[2..];
    let script_buf = ScriptBuf::from_hex(&script).unwrap();

    let pubkey = script_buf.p2pk_public_key().unwrap();
    // let pubkey_hash = pubkey.pubkey_hash();
    let addr = Address::p2pkh(&pubkey, bitcoin::Network::Bitcoin);
    println!("{}, {}", addr, serde_json::to_string(&addr).unwrap());
  }

  #[test]
  fn test_from_std_script() {
    let script = "00143be88101ba8d429dfbac9df009a445b51ccf1745";
    let script_buf = ScriptBuf::from_hex(&script).unwrap();
    println!(
      "{:?},{}",
      script_buf.is_p2pk(),
      Address::from_script(&script_buf, bitcoin::Network::Bitcoin).unwrap()
    );
  }
}
