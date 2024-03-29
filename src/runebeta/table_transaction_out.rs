use bitcoin::TxIn;
use diesel::{associations::HasTable, ExpressionMethods, PgConnection, RunQueryDsl};

use super::models::NewTransactionOut;
use crate::schema::transaction_outs::dsl::*;

#[derive(Clone)]
pub struct TransactionOutTable {}

impl<'conn> TransactionOutTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn inserts(
    &self,
    txs: &Vec<NewTransactionOut>,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_outs::table())
      .values(txs)
      .execute(connection)
  }
  //Run in the same transaction as txin indexing
  pub fn spends(
    &self,
    txins: &Vec<&TxIn>,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    for txin in txins.iter() {
      diesel::update(transaction_outs)
        .filter(tx_hash.eq(txin.previous_output.txid.to_string().as_str()))
        .filter(vout.eq(txin.previous_output.vout as i64))
        .set(spent.eq(true))
        .execute(connection)?;
    }
    Ok(txins.len())
  }

  pub fn spend(
    &self,
    txins: &Vec<TxIn>,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    for txin in txins.iter() {
      diesel::update(transaction_outs)
        .filter(tx_hash.eq(txin.previous_output.txid.to_string().as_str()))
        .filter(vout.eq(txin.previous_output.vout as i64))
        .set(spent.eq(true))
        .execute(connection)?;
    }
    Ok(txins.len())
  }
}
