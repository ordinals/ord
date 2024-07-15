use super::*;
use crate::index::{RichAddress, UTXO_INDEX_TABLE};
use bitcoin::PublicKey;
use redb::Database;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Parser)]
pub(crate) struct MakeUTXOIdx {
  #[arg(long, help = "utxo list")]
  input: String,
  #[arg(long, help = "utxo:address map index")]
  output: String,
}

impl MakeUTXOIdx {
  pub(crate) fn run(self) -> SubcommandResult {
    let utxo_idx = Database::create(self.output.as_str()).unwrap();
    let write_txn = utxo_idx.begin_write().unwrap();

    let mut csv_reader = BufReader::with_capacity(8 * 1024 * 1024, File::open(self.input).unwrap());
    let mut is_title_line = true;

    {
      let mut table = write_txn.open_table(UTXO_INDEX_TABLE).unwrap();
      for line in csv_reader.by_ref().lines() {
        let line = line.unwrap();
        if is_title_line {
          is_title_line = false;
          if line.starts_with("count") {
            continue;
          }
        }
        let (utxo, rich_address) = gen_utxo_data_from_csv_line(line.as_str());
        table
          .insert(
            bcs::to_bytes(&utxo).unwrap().as_slice(),
            bcs::to_bytes(&rich_address).unwrap().as_slice(),
          )
          .unwrap();
      }
    }

    write_txn.commit().unwrap();
    Ok(None)
  }
}

// csv format: count,txid,vout,height,coinbase,amount,script,type,address
fn gen_utxo_data_from_csv_line(line: &str) -> (String, RichAddress) {
  let str_list: Vec<&str> = line.trim().split(',').collect();
  if str_list.len() != 9 {
    panic!("Invalid utxo data: {}", line);
  }
  let txid = str_list[1].to_string();
  let vout = str_list[2].parse::<u32>().expect("Invalid vout format");
  let output = format!("{}:{}", txid, vout);

  let mut rich_address = RichAddress {
    script_type: str_list[7].to_string(),
    address: if str_list[8].is_empty() {
      let script = str_list[6].to_string();
      let script_buf = ScriptBuf::from_hex(&script).unwrap();
      match Address::from_script(&script_buf, bitcoin::Network::Bitcoin) {
        Ok(address) => Some(address.to_string()),
        Err(_) => None,
      }
    } else {
      Some(str_list[8].to_string())
    },
  };
  if rich_address.script_type == "p2pk" {
    let script = str_list[6].to_string();
    if let Ok(pubkey) = PublicKey::from_str(script.as_str()) {
      // there are invalid p2pk scripts
      rich_address.address = Some(pubkey.to_string());
    }
  };

  (output, rich_address)
}
