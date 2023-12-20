use {
  super::*,
  crate::{
    index::OUTPOINT_TO_ENTRY,
    okx::datastore::ord::{DataStoreReadOnly, DataStoreReadWrite, InscriptionOp},
    InscriptionId, Result,
  },
  bitcoin::{consensus::Encodable, OutPoint, TxOut, Txid},
  redb::{ReadTransaction, WriteTransaction},
};

pub fn try_init_tables<'db, 'a>(
  wtx: &'a WriteTransaction<'db>,
  rtx: &'a ReadTransaction<'db>,
) -> Result<bool, redb::Error> {
  if rtx.open_table(ORD_TX_TO_OPERATIONS).is_err() {
    wtx.open_table(ORD_TX_TO_OPERATIONS)?;
    wtx.open_table(COLLECTIONS_KEY_TO_INSCRIPTION_ID)?;
    wtx.open_table(COLLECTIONS_INSCRIPTION_ID_TO_KINDS)?;
  }
  Ok(true)
}

pub struct OrdDbReadWriter<'db, 'a> {
  wtx: &'a WriteTransaction<'db>,
}

impl<'db, 'a> OrdDbReadWriter<'db, 'a>
where
  'db: 'a,
{
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self { wtx }
  }
}

impl<'db, 'a> DataStoreReadOnly for OrdDbReadWriter<'db, 'a> {
  type Error = redb::Error;
  fn get_number_by_inscription_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<i32>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_number_by_inscription_id(inscription_id)
  }

  fn get_outpoint_to_txout(&self, outpoint: OutPoint) -> Result<Option<TxOut>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_outpoint_to_txout(outpoint)
  }

  fn get_transaction_operations(
    &self,
    txid: &bitcoin::Txid,
  ) -> Result<Vec<InscriptionOp>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_transaction_operations(txid)
  }
  // collections
  fn get_collection_inscription_id(&self, key: &str) -> Result<Option<InscriptionId>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_collection_inscription_id(key)
  }
  fn get_collections_of_inscription(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<Vec<CollectionKind>>, Self::Error> {
    read_only::new_with_wtx(self.wtx).get_collections_of_inscription(inscription_id)
  }
}

impl<'db, 'a> DataStoreReadWrite for OrdDbReadWriter<'db, 'a> {
  // OUTPOINT_TO_SCRIPT

  fn set_outpoint_to_txout(&self, outpoint: OutPoint, tx_out: &TxOut) -> Result<(), Self::Error> {
    let mut value = [0; 36];
    outpoint
      .consensus_encode(&mut value.as_mut_slice())
      .unwrap();

    let mut entry = Vec::new();
    tx_out.consensus_encode(&mut entry)?;
    self
      .wtx
      .open_table(OUTPOINT_TO_ENTRY)?
      .insert(&value, entry.as_slice())?;
    Ok(())
  }

  fn save_transaction_operations(
    &self,
    txid: &Txid,
    operations: &[InscriptionOp],
  ) -> Result<(), Self::Error> {
    self.wtx.open_table(ORD_TX_TO_OPERATIONS)?.insert(
      txid.to_string().as_str(),
      bincode::serialize(operations).unwrap().as_slice(),
    )?;
    Ok(())
  }
  fn set_inscription_by_collection_key(
    &self,
    key: &str,
    inscription_id: InscriptionId,
  ) -> Result<(), Self::Error> {
    let mut value = [0; 36];
    let (txid, index) = value.split_at_mut(32);
    txid.copy_from_slice(inscription_id.txid.as_ref());
    index.copy_from_slice(&inscription_id.index.to_be_bytes());
    self
      .wtx
      .open_table(COLLECTIONS_KEY_TO_INSCRIPTION_ID)?
      .insert(key, &value)?;
    Ok(())
  }

  fn set_inscription_attributes(
    &self,
    inscription_id: InscriptionId,
    kind: &[CollectionKind],
  ) -> Result<(), Self::Error> {
    let mut key = [0; 36];
    let (txid, index) = key.split_at_mut(32);
    txid.copy_from_slice(inscription_id.txid.as_ref());
    index.copy_from_slice(&inscription_id.index.to_be_bytes());
    self
      .wtx
      .open_table(COLLECTIONS_INSCRIPTION_ID_TO_KINDS)?
      .insert(&key, bincode::serialize(&kind).unwrap().as_slice())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{inscription, okx::datastore::ord::Action, unbound_outpoint, SatPoint};
  use redb::Database;
  use std::str::FromStr;
  use tempfile::NamedTempFile;

  #[test]
  fn test_outpoint_to_script() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let ord_db = OrdDbReadWriter::new(&wtx);

    let outpoint1 = unbound_outpoint();
    let tx_out = TxOut {
      value: 100,
      script_pubkey: bitcoin::Address::from_str("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")
        .unwrap()
        .assume_checked()
        .script_pubkey(),
    };

    ord_db.set_outpoint_to_txout(outpoint1, &tx_out).unwrap();

    assert_eq!(
      ord_db.get_outpoint_to_txout(outpoint1).unwrap().unwrap(),
      tx_out
    );
  }

  #[test]
  fn test_transaction_to_operations() {
    let dbfile = NamedTempFile::new().unwrap();
    let db = Database::create(dbfile.path()).unwrap();
    let wtx = db.begin_write().unwrap();
    let ord_db = OrdDbReadWriter::new(&wtx);
    let txid =
      Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();
    let operation = InscriptionOp {
      txid,
      action: Action::New {
        cursed: false,
        unbound: false,
        inscription: inscription("text/plain;charset=utf-8", "foobar"),
      },
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

    ord_db
      .save_transaction_operations(&txid, &[operation.clone()])
      .unwrap();

    assert_eq!(
      ord_db.get_transaction_operations(&txid).unwrap(),
      vec![operation]
    );
  }
}
