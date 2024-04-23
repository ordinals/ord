use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use crate::schema::txid_rune_addresss::dsl::*;
use crate::InsertRecords;

use super::models::NewTransactionRuneAddress;
pub const NUMBER_OF_FIELDS: u16 = 7;
#[derive(Clone)]
pub struct TransactionRuneAddressTable {}

impl TransactionRuneAddressTable {
  pub fn new() -> Self {
    Self {}
  }
}

impl InsertRecords for TransactionRuneAddressTable {
  const TABLE_NAME: &'static str = "tx_rune_address";
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewTransactionRuneAddress;

  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(txid_rune_addresss::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }

  fn insert_record(
    &self,
    records: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(txid_rune_addresss::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
