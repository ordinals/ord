use std::thread::{self, JoinHandle};
use std::time::Instant;

use diesel::associations::HasTable;
use diesel::query_builder::SqlQuery;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{PgConnection, RunQueryDsl};

use crate::schema::rune_stats::dsl::*;
use crate::{calculate_chunk_size, split_input, InsertRecords};

use super::models::NewRuneStats;
use super::table_transaction_rune_entry::create_update_rune_mintable;
pub const NUMBER_OF_FIELDS: u16 = 8;

pub fn create_update_rune_entry(height: &u64) -> SqlQuery {
  let query = format!(
    r#"
      UPDATE transaction_rune_entries e SET 
        mints = e.mints + s.mints, 
        supply = e.supply + s.mint_amount * s.mints,
        burned = e.burned + s.burned,  
        remaining = e.remaining - s.mints
        total_tx_count = e.total_tx_count + s.tx_count
      FROM rune_stats s
      WHERE s.block_height = {} 
      AND s.aggregated = false
      AND e.rune_id = s.rune_id;
      "#,
    height
  );
  diesel::sql_query(query)
}

pub fn create_update_rune_stats(heights: &Vec<u64>) -> SqlQuery {
  let query = format!(
    r#"UPDATE rune_stats SET aggregated = true WHERE block_height in ({});"#,
    heights
      .iter()
      .map(|h| h.to_string())
      .collect::<Vec<String>>()
      .join(",")
  );
  diesel::sql_query(query)
}

#[derive(Clone)]
pub struct RuneStatsTable {}

impl<'conn> RuneStatsTable {
  pub fn new() -> Self {
    Self {}
  }
}
impl RuneStatsTable {
  pub fn update(
    &self,
    stats: Vec<(u64, Vec<NewRuneStats>)>,
    conn_pool: Pool<ConnectionManager<PgConnection>>,
  ) -> Result<Vec<JoinHandle<()>>, diesel::result::Error> {
    let mut heights = vec![];
    let mut records = vec![];
    for (height, mut stat) in stats.into_iter() {
      heights.push(height);
      records.append(&mut stat);
    }
    let handle = thread::spawn(move || {
      //1. Adjust chunk_size for split vector into chunk with equals length
      let chunk_size = calculate_chunk_size(records.len(), Self::CHUNK_SIZE);
      //2. Break input records into chunks
      let chunks = split_input(records, chunk_size);
      //3. In eacch iteration we get first Self::CHUNK_SIZE in remain vector then put into a single insert query
      //Loop until we success get connection from the pool
      loop {
        if let Ok(mut conn) = conn_pool.get() {
          let start = Instant::now();
          for chunk in chunks {
            match Self::insert_slice(&chunk, &mut conn) {
              Ok(_) => log::info!(
                "Inserted {} records into table {} in {} ms",
                chunk.len(),
                Self::TABLE_NAME,
                start.elapsed().as_millis()
              ),
              Err(err) => log::info!("Insert stat error {:?}", err),
            }
          }
          //After success insert, run update to the rune entry

          let _res = conn.build_transaction().read_write().run(|conn| {
            let mut size = 0_usize;
            for height in heights.iter() {
              let update_rune_entries = create_update_rune_entry(height);
              let rune_res = update_rune_entries.execute(conn);
              match &rune_res {
                Ok(res) => {
                  log::info!(
                    "Update rune entry for block {} in {} ms",
                    height,
                    start.elapsed().as_millis()
                  );
                  size = size + res;
                }
                Err(err) => {
                  log::info!("Update rune entries error {:?}", err);
                  return rune_res;
                }
              }
            }
            let update_rune_stat = create_update_rune_stats(&heights);
            let stat_res = update_rune_stat.execute(conn);
            match &stat_res {
              Ok(_) => log::info!(
                "Updated rune stats for blocks {:?} in {} ms",
                heights,
                start.elapsed().as_millis()
              ),
              Err(err) => {
                log::info!("Update stats error {:?}", err);
                return stat_res;
              }
            };
            //Update mintable by execute query with latest heights
            if let Some(height) = heights.last() {
              let update_rune_stat = create_update_rune_mintable(height);
              let stat_res = update_rune_stat.execute(conn);
              match &stat_res {
                Ok(_) => log::info!(
                  "Updated rune mintable for block {:?} in {} ms",
                  heights,
                  start.elapsed().as_millis()
                ),
                Err(err) => {
                  log::info!("Update stats error {:?}", err);
                  return stat_res;
                }
              };
            }

            Ok(size)
          });
          break;
        }
      }
    });
    Ok(vec![handle])
  }
}

impl InsertRecords for RuneStatsTable {
  const TABLE_NAME: &'static str = "rune_stats";
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewRuneStats;
  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(rune_stats::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }

  fn insert_record(
    &self,
    record: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(rune_stats::table())
      .values(record)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
