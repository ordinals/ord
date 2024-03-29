use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use crate::runebeta::models::NewTransactionIn;
use crate::schema::transaction_ins::dsl::*;
#[derive(Clone)]
pub struct TransactionInTable {}

impl<'conn> TransactionInTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn inserts(
    &self,
    txs: &Vec<NewTransactionIn>,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_ins::table())
      .values(txs)
      .execute(connection)
  }
}
