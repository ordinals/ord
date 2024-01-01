use crate::index::entry::Entry;
use crate::index::{InscriptionEntryValue, InscriptionIdValue, OutPointValue, TxidValue};
use crate::inscription_id::InscriptionId;
use crate::okx::datastore::ord::collections::CollectionKind;
use crate::okx::datastore::ord::InscriptionOp;
use bitcoin::consensus::{Decodable, Encodable};
use bitcoin::{OutPoint, TxOut, Txid};
use redb::{ReadableTable, Table};
use std::io;

// COLLECTIONS_INSCRIPTION_ID_TO_KINDS
pub fn get_collections_of_inscription<T>(
  table: &T,
  inscription_id: &InscriptionId,
) -> crate::Result<Option<Vec<CollectionKind>>>
where
  T: ReadableTable<InscriptionIdValue, &'static [u8]>,
{
  Ok(
    table
      .get(&inscription_id.store())?
      .map(|v| rmp_serde::from_slice::<Vec<CollectionKind>>(v.value()).unwrap()),
  )
}

// COLLECTIONS_KEY_TO_INSCRIPTION_ID
pub fn get_collection_inscription_id<T>(
  table: &T,
  key: &str,
) -> crate::Result<Option<InscriptionId>>
where
  T: ReadableTable<&'static str, InscriptionIdValue>,
{
  Ok(table.get(key)?.map(|v| InscriptionId::load(v.value())))
}

// SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY
pub fn get_inscription_number_by_sequence_number<T>(
  table: &T,
  sequence_number: u32,
) -> crate::Result<Option<i32>>
where
  T: ReadableTable<u32, InscriptionEntryValue>,
{
  Ok(table.get(sequence_number)?.map(|value| value.value().4))
}

// OUTPOINT_TO_ENTRY
pub fn get_txout_by_outpoint<T>(table: &T, outpoint: &OutPoint) -> crate::Result<Option<TxOut>>
where
  T: ReadableTable<&'static OutPointValue, &'static [u8]>,
{
  let mut value = [0; 36];
  outpoint
    .consensus_encode(&mut value.as_mut_slice())
    .unwrap();
  Ok(
    table
      .get(&value)?
      .map(|x| Decodable::consensus_decode(&mut io::Cursor::new(x.value())).unwrap()),
  )
}

// ORD_TX_TO_OPERATIONS
pub fn get_transaction_operations<T>(table: &T, txid: &Txid) -> crate::Result<Vec<InscriptionOp>>
where
  T: ReadableTable<&'static TxidValue, &'static [u8]>,
{
  Ok(table.get(&txid.store())?.map_or(Vec::new(), |v| {
    rmp_serde::from_slice::<Vec<InscriptionOp>>(v.value()).unwrap()
  }))
}

// ORD_TX_TO_OPERATIONS
pub fn save_transaction_operations<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static TxidValue, &'static [u8]>,
  txid: &Txid,
  operations: &[InscriptionOp],
) -> crate::Result<()> {
  table.insert(&txid.store(), rmp_serde::to_vec(operations)?.as_slice())?;
  Ok(())
}

// COLLECTIONS_KEY_TO_INSCRIPTION_ID
pub fn set_inscription_by_collection_key<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, InscriptionIdValue>,
  key: &str,
  inscription_id: &InscriptionId,
) -> crate::Result<()> {
  table.insert(key, inscription_id.store())?;
  Ok(())
}

// COLLECTIONS_INSCRIPTION_ID_TO_KINDS
pub fn set_inscription_attributes<'db, 'txn>(
  table: &mut Table<'db, 'txn, InscriptionIdValue, &'static [u8]>,
  inscription_id: &InscriptionId,
  kind: &[CollectionKind],
) -> crate::Result<()> {
  table.insert(
    inscription_id.store(),
    rmp_serde::to_vec(&kind).unwrap().as_slice(),
  )?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::index::ORD_TX_TO_OPERATIONS;
  use crate::okx::datastore::ord::redb::table::{
    get_transaction_operations, save_transaction_operations,
  };
  use crate::okx::datastore::ord::InscriptionOp;
  use crate::{inscription, okx::datastore::ord::Action, SatPoint};
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;

  #[test]
  fn test_transaction_to_operations() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let mut table = wtx.open_table(ORD_TX_TO_OPERATIONS).unwrap();
    let txid =
      Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();
    let operation = InscriptionOp {
      txid,
      action: Action::New {
        cursed: false,
        unbound: false,
        inscription: inscription("text/plain;charset=utf-8", "foobar"),
      },
      sequence_number: 100,
      inscription_number: Some(100),
      inscription_id: InscriptionId { txid, index: 0 },
      old_satpoint: SatPoint::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111:1:1",
      )
      .unwrap(),
      new_satpoint: Some(SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 1,
      }),
    };

    save_transaction_operations(&mut table, &txid, &[operation.clone()]).unwrap();

    assert_eq!(
      get_transaction_operations(&table, &txid).unwrap(),
      vec![operation]
    );
  }
}
