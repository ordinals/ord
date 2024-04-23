use diesel::{associations::HasTable, PgConnection, RunQueryDsl, SelectableHelper};

use super::models::{NewTransaction, Transaction};
use crate::{schema::transactions::dsl::*, InsertRecords};
pub const NUMBER_OF_FIELDS: u16 = 6;
#[derive(Clone)]
pub struct TransactionTable {}

impl<'conn> TransactionTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn _insert(
    &self,
    tx: &NewTransaction,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transactions::table())
      .values(tx)
      .on_conflict(tx_hash)
      .do_update()
      .set(tx)
      .returning(Transaction::as_returning())
      .execute(connection)
  }
  // pub fn inserts(
  //   &self,
  //   txs: &[NewTransaction],
  //   connection: &mut PgConnection,
  // ) -> Result<usize, diesel::result::Error> {
  //   diesel::insert_into(transactions::table())
  //     .values(txs)
  //     .on_conflict_do_nothing()
  //     .returning(Transaction::as_returning())
  //     .execute(connection)
  // }
}

impl InsertRecords for TransactionTable {
  const TABLE_NAME: &'static str = "transactions";
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewTransaction;
  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transactions::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }

  fn insert_record(
    &self,
    record: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transactions::table())
      .values(record)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
