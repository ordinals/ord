use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use crate::runebeta::models::NewTransactionIn;
use crate::schema::transaction_ins::dsl::*;
use crate::InsertRecords;
pub const NUMBER_OF_FIELDS: u16 = 10;
#[derive(Clone)]
pub struct TransactionInTable {}

impl<'conn> TransactionInTable {
  pub fn new() -> Self {
    Self {}
  }
}

impl InsertRecords for TransactionInTable {
  const TABLE_NAME: &'static str = "transaction_ins";
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewTransactionIn;
  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_ins::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }

  fn insert_record(
    &self,
    record: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_ins::table())
      .values(record)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
