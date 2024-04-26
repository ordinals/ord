use bigdecimal::BigDecimal;
use diesel::query_builder::SqlQuery;
use diesel::{associations::HasTable, PgConnection, RunQueryDsl};

use crate::schema::transaction_rune_entries::dsl::*;
use crate::{InsertRecords, RuneEntry, RuneId};

use super::models::{NewTxRuneEntry, RuneTerms};
pub const NUMBER_OF_FIELDS: u16 = 30;
pub const RUNE_MINT_TYPE_FIXED_CAP: &str = "fixed-cap";
pub const RUNE_MINT_TYPE_FAIRMINT: &str = "fairmint";

pub fn create_update_rune_mintable(height: &u64) -> SqlQuery {
  //
  let query = format!(
    r#"
    UPDATE transaction_rune_entries 
      SET mintable = COALESCE (offset_start , -block_height) + block_height <= {0} 
            AND COALESCE (height_start, 0) <= {0}
      AND COALESCE (offset_end , {0} - block_height) + block_height >= {0}  
      AND COALESCE(height_end, {0} ) >= {0} 
      AND cap > mints
    WHERE terms IS NOT NULL AND ((mintable OR (IS NOT mintable AND (mints <= cap)));"#,
    height
  );
  diesel::sql_query(query)
}

pub fn create_update_rune_total_holders() -> SqlQuery {
  let query = r"
  WITH rune_total_holders AS (
    SELECT COUNT(DISTINCT address) AS total_holders, rune_id
    FROM outpoint_rune_balances
    WHERE balance_value > 0
    GROUP BY rune_id
  )
  UPDATE transaction_rune_entries
  SET total_holders = rtd.total_holders
  FROM rune_total_holders rtd
  WHERE transaction_rune_entries.rune_id = rtd.rune_id;
  ";
  diesel::sql_query(query)
}

#[derive(Clone)]
pub struct TransactionRuneEntryTable {}

impl<'conn> TransactionRuneEntryTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn create(
    &self,
    rune_id_value: &RuneId,
    rune_entry: &RuneEntry,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    let mut tx_rune_entry = NewTxRuneEntry::from(rune_entry);
    tx_rune_entry.tx_index = rune_id_value.tx as i32;
    tx_rune_entry.rune_id = rune_id_value.to_string();
    diesel::insert_into(transaction_rune_entries::table())
      .values(tx_rune_entry)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
/*
 * missing tx_index, rune_id
 */
impl From<&RuneEntry> for NewTxRuneEntry {
  fn from(rune_entry: &RuneEntry) -> Self {
    let mint_remain = rune_entry
      .terms
      .and_then(|rune_terms| {
        rune_terms
          .cap
          .as_ref()
          .map(|v| BigDecimal::from(v - rune_entry.mints))
      })
      .unwrap_or_else(|| BigDecimal::from(0));
    NewTxRuneEntry {
      tx_index: 0,
      rune_id: String::default(),
      block_height: rune_entry.block as i64,
      tx_hash: rune_entry.etching.to_string(),
      burned: BigDecimal::from(rune_entry.burned),
      divisibility: rune_entry.divisibility as i16,
      etching: rune_entry.etching.to_string(),
      parent: None,
      total_tx_count: 0,
      total_holders: 0,
      mintable: rune_entry.mintable(rune_entry.block).is_ok(),
      mint_type: rune_entry.terms.map_or_else(
        || String::from(RUNE_MINT_TYPE_FIXED_CAP),
        |_| String::from(RUNE_MINT_TYPE_FAIRMINT),
      ),
      mints: rune_entry.mints as i64,
      number: rune_entry.number as i64,
      rune: BigDecimal::from(rune_entry.spaced_rune.rune.0),
      spacers: rune_entry.spaced_rune.spacers as i32,
      premine: BigDecimal::from(rune_entry.premine),
      remaining: mint_remain,
      spaced_rune: rune_entry.spaced_rune.to_string(),
      supply: BigDecimal::from(rune_entry.premine),
      symbol: rune_entry.symbol.map(|c| c.to_string()),
      timestamp: rune_entry.timestamp as i32,
      terms: rune_entry.terms.map(|entry| RuneTerms::from(&entry)),
      height_start: rune_entry
        .terms
        .and_then(|entry| entry.height.0.as_ref().map(|v| v.clone() as i64)),
      height_end: rune_entry
        .terms
        .and_then(|entry| entry.height.1.as_ref().map(|v| v.clone() as i64)),
      offset_start: rune_entry
        .terms
        .and_then(|entry| entry.offset.0.as_ref().map(|v| v.clone() as i64)),
      offset_end: rune_entry
        .terms
        .and_then(|entry| entry.offset.1.as_ref().map(|v| v.clone() as i64)),
      cap: rune_entry
        .terms
        .and_then(|entry| entry.cap.as_ref().map(|v| BigDecimal::from(v)))
        .unwrap_or_default(),
      turbo: rune_entry.turbo,
    }
  }
}

impl InsertRecords for TransactionRuneEntryTable {
  const TABLE_NAME: &'static str = "transaction_rune_entry";
  const CHUNK_SIZE: usize = (u16::MAX / NUMBER_OF_FIELDS) as usize;
  type Record = NewTxRuneEntry;
  fn insert_slice(
    records: &[Self::Record],
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_rune_entries::table())
      .values(records)
      .on_conflict_do_nothing()
      .execute(connection)
  }

  fn insert_record(
    &self,
    record: &Self::Record,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(transaction_rune_entries::table())
      .values(record)
      .on_conflict_do_nothing()
      .execute(connection)
  }
}
