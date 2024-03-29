use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use crate::runebeta::models::TransactionRune;
use crate::schema::txid_runes::dsl::*;
#[derive(Clone)]
pub struct TransactionRuneTable {}

impl<'conn> TransactionRuneTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn insert(
    &self,
    txs: &Vec<TransactionRune>,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(txid_runes::table())
      .values(txs)
      .execute(connection)
  }
}
