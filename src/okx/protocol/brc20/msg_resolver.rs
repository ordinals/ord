use super::*;
use crate::index::InscriptionIdValue;
use crate::okx::datastore::brc20::redb::table::get_inscribe_transfer_inscription;
use crate::{
  inscription::Inscription,
  okx::{
    datastore::ord::{Action, InscriptionOp},
    protocol::brc20::{deserialize_brc20_operation, Operation},
  },
  Result,
};
use anyhow::anyhow;
use redb::ReadableTable;

impl Message {
  pub(crate) fn resolve<T>(
    table: &T,
    new_inscriptions: &[Inscription],
    op: &InscriptionOp,
  ) -> Result<Option<Message>>
  where
    T: ReadableTable<InscriptionIdValue, &'static [u8]>,
  {
    log::debug!("BRC20 resolving the message from {:?}", op);
    let sat_in_outputs = op
      .new_satpoint
      .map(|satpoint| satpoint.outpoint.txid == op.txid)
      .unwrap_or(false);

    let brc20_operation = match op.action {
      // New inscription is not `cursed` or `unbound`.
      Action::New {
        cursed: false,
        unbound: false,
        inscription: _,
      } if sat_in_outputs => {
        match deserialize_brc20_operation(
          new_inscriptions
            .get(usize::try_from(op.inscription_id.index).unwrap())
            .unwrap(),
          &op.action,
        ) {
          Ok(brc20_operation) => brc20_operation,
          _ => return Ok(None),
        }
      }
      // Transfered inscription operation.
      // Attempt to retrieve the `InscribeTransfer` Inscription information from the data store of BRC20S.
      Action::Transfer => match get_inscribe_transfer_inscription(table, &op.inscription_id) {
        // Ignore non-first transfer operations.
        Ok(Some(transfer_info)) if op.inscription_id.txid == op.old_satpoint.outpoint.txid => {
          Operation::Transfer(Transfer {
            tick: transfer_info.tick.as_str().to_string(),
            amount: transfer_info.amt.to_string(),
          })
        }
        Err(e) => {
          return Err(anyhow!(
            "failed to get inscribe transfer inscription for {}! error: {e}",
            op.inscription_id,
          ))
        }
        _ => return Ok(None),
      },
      _ => return Ok(None),
    };
    Ok(Some(Self {
      txid: op.txid,
      sequence_number: op.sequence_number,
      inscription_id: op.inscription_id,
      old_satpoint: op.old_satpoint,
      new_satpoint: op.new_satpoint,
      op: brc20_operation,
      sat_in_outputs,
    }))
  }
}
// #[cfg(test)]
// mod tests {
//   use super::*;
//   use crate::okx::datastore::brc20::{Brc20ReaderWriter, Tick, TransferInfo};
//   use bitcoin::OutPoint;
//   use redb::Database;
//   use std::str::FromStr;
//   use tempfile::NamedTempFile;
//   fn create_inscription(str: &str) -> Inscription {
//     Inscription::new(
//       Some("text/plain;charset=utf-8".as_bytes().to_vec()),
//       Some(str.as_bytes().to_vec()),
//     )
//   }
//
//   fn create_inscribe_operation(str: &str) -> (Vec<Inscription>, InscriptionOp) {
//     let inscriptions = vec![create_inscription(str)];
//     let txid =
//       Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();
//     let op = InscriptionOp {
//       txid,
//       action: Action::New {
//         cursed: false,
//         unbound: false,
//         inscription: inscriptions.get(0).unwrap().clone(),
//       },
//       inscription_number: Some(1),
//       inscription_id: InscriptionId { txid, index: 0 },
//       old_satpoint: SatPoint {
//         outpoint: OutPoint {
//           txid: Txid::from_str("2111111111111111111111111111111111111111111111111111111111111111")
//             .unwrap(),
//           vout: 0,
//         },
//         offset: 0,
//       },
//       new_satpoint: Some(SatPoint {
//         outpoint: OutPoint { txid, vout: 0 },
//         offset: 0,
//       }),
//     };
//     (inscriptions, op)
//   }
//
//   fn create_transfer_operation() -> InscriptionOp {
//     let txid =
//       Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();
//
//     let inscription_id = InscriptionId {
//       txid: Txid::from_str("2111111111111111111111111111111111111111111111111111111111111111")
//         .unwrap(),
//       index: 0,
//     };
//
//     InscriptionOp {
//       txid,
//       action: Action::Transfer,
//       inscription_number: Some(1),
//       inscription_id,
//       old_satpoint: SatPoint {
//         outpoint: OutPoint {
//           txid: inscription_id.txid,
//           vout: 0,
//         },
//         offset: 0,
//       },
//       new_satpoint: Some(SatPoint {
//         outpoint: OutPoint { txid, vout: 0 },
//         offset: 0,
//       }),
//     }
//   }
//
//   #[test]
//   fn test_invalid_protocol() {
//     let db_file = NamedTempFile::new().unwrap();
//     let db = Database::create(db_file.path()).unwrap();
//     let wtx = db.begin_write().unwrap();
//     let brc20_store = DataStore::new(&wtx);
//
//     let (inscriptions, op) = create_inscribe_operation(
//       r#"{ "p": "brc-20s","op": "deploy", "tick": "ordi", "max": "1000", "lim": "10" }"#,
//     );
//     assert_matches!(Message::resolve(&brc20_store, &inscriptions, &op), Ok(None));
//   }
//
//   #[test]
//   fn test_cursed_or_unbound_inscription() {
//     let db_file = NamedTempFile::new().unwrap();
//     let db = Database::create(db_file.path()).unwrap();
//     let wtx = db.begin_write().unwrap();
//     let brc20_store = DataStore::new(&wtx);
//
//     let (inscriptions, op) = create_inscribe_operation(
//       r#"{ "p": "brc-20","op": "deploy", "tick": "ordi", "max": "1000", "lim": "10" }"#,
//     );
//     let op = InscriptionOp {
//       action: Action::New {
//         cursed: true,
//         unbound: false,
//         inscription: inscriptions.get(0).unwrap().clone(),
//       },
//       ..op
//     };
//     assert_matches!(Message::resolve(&brc20_store, &inscriptions, &op), Ok(None));
//
//     let op2 = InscriptionOp {
//       action: Action::New {
//         cursed: false,
//         unbound: true,
//         inscription: inscriptions.get(0).unwrap().clone(),
//       },
//       ..op
//     };
//     assert_matches!(
//       Message::resolve(&brc20_store, &inscriptions, &op2),
//       Ok(None)
//     );
//     let op3 = InscriptionOp {
//       action: Action::New {
//         cursed: true,
//         unbound: true,
//         inscription: inscriptions.get(0).unwrap().clone(),
//       },
//       ..op
//     };
//     assert_matches!(
//       Message::resolve(&brc20_store, &inscriptions, &op3),
//       Ok(None)
//     );
//   }
//
//   #[test]
//   fn test_valid_inscribe_operation() {
//     let db_file = NamedTempFile::new().unwrap();
//     let db = Database::create(db_file.path()).unwrap();
//     let wtx = db.begin_write().unwrap();
//     let brc20_store = DataStore::new(&wtx);
//
//     let (inscriptions, op) = create_inscribe_operation(
//       r#"{ "p": "brc-20","op": "deploy", "tick": "ordi", "max": "1000", "lim": "10" }"#,
//     );
//     let _result_msg = Message {
//       txid: op.txid,
//       inscription_id: op.inscription_id,
//       old_satpoint: op.old_satpoint,
//       new_satpoint: op.new_satpoint,
//       op: Operation::Deploy(Deploy {
//         tick: "ordi".to_string(),
//         max_supply: "1000".to_string(),
//         mint_limit: Some("10".to_string()),
//         decimals: None,
//       }),
//       sat_in_outputs: true,
//     };
//     assert_matches!(
//       Message::resolve(&brc20_store, &inscriptions, &op),
//       Ok(Some(_result_msg))
//     );
//   }
//
//   #[test]
//   fn test_invalid_transfer() {
//     let db_file = NamedTempFile::new().unwrap();
//     let db = Database::create(db_file.path()).unwrap();
//     let wtx = db.begin_write().unwrap();
//     let brc20_store = DataStore::new(&wtx);
//
//     // inscribe transfer not found
//     let op = create_transfer_operation();
//     assert_matches!(Message::resolve(&brc20_store, &[], &op), Ok(None));
//
//     // non-first transfer operations.
//     let op1 = InscriptionOp {
//       old_satpoint: SatPoint {
//         outpoint: OutPoint {
//           txid: Txid::from_str("3111111111111111111111111111111111111111111111111111111111111111")
//             .unwrap(),
//           vout: 0,
//         },
//         offset: 0,
//       },
//       ..op
//     };
//     assert_matches!(Message::resolve(&brc20_store, &[], &op1), Ok(None));
//   }
//
//   #[test]
//   fn test_valid_transfer() {
//     let db_file = NamedTempFile::new().unwrap();
//     let db = Database::create(db_file.path()).unwrap();
//     let wtx = db.begin_write().unwrap();
//     let brc20_store = DataStore::new(&wtx);
//
//     // inscribe transfer not found
//     let op = create_transfer_operation();
//
//     brc20_store
//       .insert_inscribe_transfer_inscription(
//         op.inscription_id,
//         TransferInfo {
//           tick: Tick::from_str("ordi").unwrap(),
//           amt: 100,
//         },
//       )
//       .unwrap();
//     let _msg = Message {
//       txid: op.txid,
//       inscription_id: op.inscription_id,
//       old_satpoint: op.old_satpoint,
//       new_satpoint: op.new_satpoint,
//       op: Operation::Transfer(Transfer {
//         tick: "ordi".to_string(),
//         amount: "100".to_string(),
//       }),
//       sat_in_outputs: true,
//     };
//
//     assert_matches!(Message::resolve(&brc20_store, &[], &op), Ok(Some(_msg)));
//   }
// }
