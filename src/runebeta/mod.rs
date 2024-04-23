pub mod extension;
mod models;
pub mod schema;
mod table_block;
mod table_outpoint_rune_balance;
mod table_rune_stats;
mod table_transaction;
mod table_transaction_in;
mod table_transaction_out;
mod table_transaction_rune;
mod table_transaction_rune_address;
mod table_transaction_rune_entry;
use anyhow::anyhow;
use diesel::{
  query_builder::SqlQuery,
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
pub use extension::IndexExtension;
use lazy_static::lazy_static;
use std::{
  env,
  fmt::Debug,
  thread::{self, JoinHandle},
  time::Instant,
};
pub use table_block::BlockTable;
pub use table_outpoint_rune_balance::OutpointRuneBalanceTable;
pub use table_rune_stats::RuneStatsTable;
pub use table_transaction_in::TransactionInTable;
pub use table_transaction_out::TransactionOutTable;
pub use table_transaction_rune::TransactionRuneTable;
pub use table_transaction_rune_address::TransactionRuneAddressTable;
pub use table_transaction_rune_entry::TransactionRuneEntryTable;
#[cfg(test)]
mod testing;
const SAFE_RECORDS_NUMBER_PER_QUERY: u32 = 1024;
lazy_static! {
  static ref CONNECTION_POOL_SIZE: u32 = env::var("ORD_SUPERSATS_CONNECTION_POOL_SIZE")
    .map_err(|err| anyhow!(err))
    .and_then(|size| size.parse::<u32>().map_err(|err| anyhow!(err)))
    .unwrap_or(10);
  // static ref POOL: Pool<ConnectionManager<PgConnection>> = {
  //   let db_url = env::var("DATABASE_URL").expect("Database url not set");
  //   let manager = ConnectionManager::<PgConnection>::new(db_url);
  //   Pool::builder()
  //     .max_size(*CONNECTION_POOL_SIZE)
  //     .test_on_check_out(true)
  //     .build(manager)
  //     .expect("Failed to create db pool")
  // };
}
/*
 * https://schneide.blog/2022/11/07/copying-and-moving-rows-between-tables-in-postgresql/
 */

pub fn create_query_move_spent_transaction_outs(txout_ids: &Vec<String>) -> SqlQuery {
  let params = txout_ids
    .iter()
    .map(|id| format!("'{}'", id))
    .collect::<Vec<String>>()
    .join(",");

  let sql = format!(
    r#"
      WITH selection AS (
        DELETE FROM transaction_outs
        WHERE txout_id in ({})
        RETURNING *
      )
      INSERT INTO spent_transaction_outs
        SELECT * FROM selection
      ON CONFLICT DO NOTHING;
      "#,
    params
  );
  diesel::sql_query(sql)
}

pub fn create_query_move_outpoint_rune_balances(txout_ids: &Vec<String>) -> SqlQuery {
  let params = txout_ids
    .iter()
    .map(|id| format!("'{}'", id))
    .collect::<Vec<String>>()
    .join(",");

  let sql = format!(
    r#"
      WITH selection AS (
        DELETE FROM outpoint_rune_balances
        WHERE txout_id in ({})
        RETURNING *
      )
      INSERT INTO spent_outpoint_rune_balances
        SELECT * FROM selection
      ON CONFLICT DO NOTHING  ;
    "#,
    params
  );
  diesel::sql_query(sql)
}
pub trait InsertRecords {
  const CHUNK_SIZE: usize;
  const TABLE_NAME: &'static str;
  type Record: 'static + Debug + Send;
  fn insert_vector(
    &self,
    records: Vec<Self::Record>,
    conn_pool: Pool<ConnectionManager<PgConnection>>,
  ) -> Result<Vec<JoinHandle<()>>, diesel::result::Error> {
    if records.len() == 0 {
      return Ok(Default::default());
    }
    //let chunks = records.chunks(Self::CHUNK_SIZE);
    //1. Adjust chunk_size for split vector into chunk with equals length
    let chunk_size = calculate_chunk_size(records.len(), Self::CHUNK_SIZE);
    //2. Break input records into chunks
    let chunks = split_input(records, chunk_size);
    //3. In eacch iteration we get first Self::CHUNK_SIZE in remain vector then put into a single insert query
    let mut handles = vec![];
    for chunk in chunks {
      let pool = conn_pool.clone();

      let handle = thread::spawn(move || {
        //Move chunk into child thread
        let thread_chunk = chunk;
        //Loop until we success get connection from the pool
        loop {
          if let Ok(mut conn) = pool.get() {
            let start = Instant::now();
            let res = Self::insert_slice(&thread_chunk, &mut conn);
            if res.is_err() {
              log::info!("Insert error {:?}", res);
            } else {
              log::info!(
                "Inserted {} records into table {} in {} ms",
                thread_chunk.len(),
                Self::TABLE_NAME,
                start.elapsed().as_millis()
              );
            }
            break;
          }
        }
      });
      handles.push(handle);
    }
    Ok(handles)
  }
  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error>;
  fn insert_record(
    &self,
    records: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error>;
}

//Calculate chunk size when device input vector
pub fn calculate_chunk_size(input_len: usize, max_size: usize) -> usize {
  //if input size is relative smalll small- we hardcode a threshold here
  if input_len == 0 {
    return input_len;
  }
  let mut required_chunks = input_len / max_size;
  if required_chunks * max_size < input_len {
    required_chunks = required_chunks + 1;
  }
  let mut number_of_chunk = required_chunks;
  let mut chunk_size = input_len / number_of_chunk;
  /*
   * We need to balance beetween number of chunk and chunk size
   * If total input is small, we use a small number of chunk
   * otherwire we increate the number of chunks for reducing the size of each chunk to a predefined value 256
   */
  while number_of_chunk < *CONNECTION_POOL_SIZE as usize
    && chunk_size > SAFE_RECORDS_NUMBER_PER_QUERY as usize
  {
    number_of_chunk = number_of_chunk + 1;
    chunk_size = input_len / number_of_chunk;
  }
  // Todo: Allocate all input_len to chunk with the same size
  chunk_size
}
/*
* Split the input vector into chunks for execute pg query
*/
pub fn split_input<T>(records: Vec<T>, chunk_size: usize) -> Vec<Vec<T>> {
  let mut output_len = 0;
  let mut chunks = vec![Vec::<T>::with_capacity(chunk_size)];
  let mut cur_chunk = chunks.get_mut(output_len).expect("Chunks mut be not empty");
  output_len = output_len + 1;
  for item in records.into_iter() {
    // Create new chunk and push to final collection if it length reaches the chunk size
    if cur_chunk.len() == chunk_size {
      chunks.push(Vec::<T>::with_capacity(chunk_size));
      //Get reference to the latest chunk;
      cur_chunk = chunks.get_mut(output_len).expect("Chunks mut be not empty");
      output_len = output_len + 1;
    }
    cur_chunk.push(item);
  }
  chunks
}
