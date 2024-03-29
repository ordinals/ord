use diesel::{associations::HasTable, PgConnection, RunQueryDsl, SelectableHelper};

use crate::{
  runebeta::models::{NewTransaction, Transaction},
  schema::transactions::dsl::*,
};

pub struct TransactionTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> TransactionTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn insert(&mut self, tx: &NewTransaction) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transactions::table())
      .values(tx)
      .on_conflict(tx_hash)
      .do_update()
      .set(tx)
      .returning(Transaction::as_returning())
      .execute(self.connection)
  }
}
