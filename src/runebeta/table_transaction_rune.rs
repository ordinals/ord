use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use crate::schema::txid_runes::dsl::*;
use crate::InsertRecords;

use super::models::NewTransactionRune;
pub const NUMBER_OF_FIELDS: u16 = 5;
#[derive(Clone)]
pub struct TransactionRuneTable {}

impl TransactionRuneTable {
  pub fn new() -> Self {
    Self {}
  }
}

impl InsertRecords for TransactionRuneTable {
  const TABLE_NAME: &'static str = "tx_rune";
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewTransactionRune;

  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(txid_runes::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }

  fn insert_record(
    &self,
    records: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(txid_runes::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
