use bitcoin::Txid;
use diesel::{associations::HasTable, PgConnection, RunQueryDsl, SelectableHelper};

use crate::{
  runebeta::models::{NewTxidRune, U128},
  schema::txid_runes::dsl::*,
};

pub struct TxidRuneTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> TxidRuneTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn insert(&self, txid: &Txid, rune_value: U128) -> Result<usize, diesel::result::Error> {
    let new_txid_rune = NewTxidRune {
      tx_hash: txid.to_raw_hash().to_string().as_str(),
      rune: rune_value,
    };
    diesel::insert_into(txid_runes::table())
      .values(&new_txid_rune)
      .on_conflict(tx_hash)
      .do_update()
      .set(&new_txid_rune)
      //.returning(TxidRune::as_returning())
      .execute(self.connection)
  }
}
