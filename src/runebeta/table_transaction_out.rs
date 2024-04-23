use std::{
  cmp,
  thread::{self, JoinHandle},
  time::Instant,
};

use super::models::NewTransactionOut;
use crate::{
  create_query_move_spent_transaction_outs, schema::transaction_outs::dsl::*, split_input,
  InsertRecords,
};

use diesel::{
  associations::HasTable,
  r2d2::{ConnectionManager, Pool},
  PgConnection, RunQueryDsl,
};
pub const NUMBER_OF_FIELDS: u16 = 18;
#[derive(Clone)]
pub struct TransactionOutTable {}

impl TransactionOutTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn inserts(
    &self,
    txs: &[NewTransactionOut],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_outs::table())
      .values(txs)
      .on_conflict_do_nothing()
      .execute(connection)
  }
  pub fn spends(
    &self,
    txins: Vec<String>,
    conn_pool: Pool<ConnectionManager<PgConnection>>,
  ) -> Result<Vec<JoinHandle<()>>, diesel::result::Error> {
    let mut handles = vec![];
    let chunk_size = cmp::min(u16::MAX as usize, txins.len());
    let chunks = split_input(txins, chunk_size);

    for chunk in chunks {
      let pool = conn_pool.clone();

      let handle = thread::spawn(move || {
        //Move chunk into child thread
        let thread_chunk = chunk;
        loop {
          if let Ok(mut connection) = pool.get() {
            let start = Instant::now();
            let query = create_query_move_spent_transaction_outs(&thread_chunk);
            let res = query.execute(&mut connection);

            // let res = diesel::update(transaction_outs)
            //   .filter(txout_id.eq_any(&thread_chunk))
            //   .set(spent.eq(true))
            //   .execute(&mut connection);

            match res {
              Ok(size) => {
                log::info!(
                  "Move out {} records from the table {} in {} ms",
                  size,
                  Self::TABLE_NAME,
                  start.elapsed().as_millis()
                );
              }
              Err(err) => {
                log::info!("Execute error {:?}", &err);
              }
            }
            break;
          }
        }
      });
      handles.push(handle);
    }
    Ok(handles)
  }
}

impl InsertRecords for TransactionOutTable {
  const TABLE_NAME: &'static str = "transaction_outs";
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewTransactionOut;
  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_outs::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }

  fn insert_record(
    &self,
    record: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_outs::table())
      .values(record)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
