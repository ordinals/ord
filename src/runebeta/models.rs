use crate::Terms;
use diesel::{
  deserialize::{FromSql, FromSqlRow},
  pg::Pg,
  prelude::*,
  serialize::{IsNull, Output, ToSql},
  sql_types::{Jsonb, Text},
  AsExpression,
};
use std::io::Write;

//https://stackoverflow.com/questions/77629993/error-extending-diesel-with-wrapper-type-for-u128
#[derive(
  Copy,
  Clone,
  FromSqlRow,
  AsExpression,
  serde::Serialize,
  serde::Deserialize,
  Debug,
  PartialEq,
  Eq,
  PartialOrd,
  Default,
)]
#[diesel(sql_type = Text)]
pub struct U128(pub u128);

impl From<u128> for U128 {
  fn from(v: u128) -> U128 {
    U128(v)
  }
}

impl From<U128> for u128 {
  fn from(v: U128) -> u128 {
    v.0
  }
}

impl ToSql<Text, Pg> for U128 {
  fn to_sql<'b>(&self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
    write!(out, "{}", self.0.to_string())?;
    Ok(IsNull::No)
  }
}
impl FromSql<Text, Pg> for U128 {
  fn from_sql(
    bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
  ) -> diesel::deserialize::Result<Self> {
    let s = String::from_utf8_lossy(bytes.as_bytes());
    Ok(U128(s.parse()?))
  }
}

// impl ToSql<Binary, Pg> for U128 {
//   fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
//     write!(out, "{}", self.0.to_ne_bytes())?;
//     Ok(IsNull::No)
//   }
// }

// impl FromSql<Binary, Pg> for U128 {
//   fn from_sql(
//     bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
//   ) -> diesel::deserialize::Result<Self> {
//     let value = u128::from_ne_bytes(*(bytes.as_bytes()));
//     Ok(U128(value))
//   }
// }

// https://vasilakisfil.social/blog/2020/05/09/rust-diesel-jsonb/

#[derive(
  Copy,
  Clone,
  FromSqlRow,
  AsExpression,
  serde::Serialize,
  serde::Deserialize,
  Debug,
  Default,
  PartialEq,
  Eq,
  PartialOrd,
)]
#[diesel(sql_type = Jsonb)]
pub struct MintEntryType {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub amount: Option<U128>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cap: Option<U128>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub height1: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub height2: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub offset1: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub offset2: Option<i64>,
}

impl ToSql<Jsonb, Pg> for MintEntryType {
  fn to_sql(&self, out: &mut Output<Pg>) -> diesel::serialize::Result {
    let value = serde_json::to_value(self)?;
    // <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
    out.write_all(&[1])?;
    serde_json::to_writer(out, &value)
      .map(|_| IsNull::No)
      .map_err(Into::into)
  }
}
impl FromSql<Jsonb, Pg> for MintEntryType {
  fn from_sql(
    bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
  ) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
    Ok(serde_json::from_value(value)?)
  }
}

impl From<&Terms> for MintEntryType {
  fn from(value: &Terms) -> Self {
    let (height1, height2) = value.height.clone();
    let (offset1, offset2) = value.offset.clone();
    MintEntryType {
      amount: value.amount.map(|v| U128(v)),
      cap: value.cap.map(|v| U128(v)),
      height1: height1.map(|v| v as i64),
      height2: height2.map(|v| v as i64),
      offset1: offset1.map(|v| v as i64),
      offset2: offset2.map(|v| v as i64),
    }
  }
}

//Block
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Block {
  pub id: i64,
  pub block_time: i64,
  pub block_height: i64,
  pub previous_hash: String,
  pub block_hash: String,
}

#[derive(AsChangeset, Insertable)]
#[diesel(table_name = crate::schema::blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewBlock {
  pub block_time: i64,
  pub block_height: i64,
  pub previous_hash: String,
  pub block_hash: String,
}

//Transaction
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Transaction {
  pub id: i64,
  pub block_height: i64,
  pub version: i32,
  pub lock_time: i32,
  pub tx_hash: String,
}

#[derive(AsChangeset, Insertable)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransaction {
  pub version: i32,
  pub block_height: i64,
  pub lock_time: i32,
  pub tx_hash: String,
}

//TransactionIn
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transaction_ins)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionIn {
  pub id: i64,
  pub tx_hash: String,
  pub previous_output_hash: String,
  pub previous_output_vout: i32,
  pub script_sig: String,
  pub sequence_number: i64,
  pub witness: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::transaction_ins)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionIn {
  pub tx_hash: String,
  pub previous_output_hash: String,
  pub previous_output_vout: i32,
  pub script_sig: String,
  pub sequence_number: i64,
  pub witness: String,
}

//TransactionOut
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transaction_outs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionOut {
  pub id: i64,
  pub tx_hash: String,
  pub vout: i64,
  pub value: i64,
  pub asm: String,
  pub dust_value: i64,
  pub address: Option<String>,
  pub script_pubkey: String,
  pub spent: bool,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::transaction_outs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionOut {
  pub tx_hash: String,
  pub vout: i64,
  pub value: i64,
  pub asm: String,
  pub dust_value: i64,
  pub address: Option<String>,
  pub script_pubkey: String,
  pub spent: bool,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::transaction_rune_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TxRuneEntry {
  pub id: i64,
  pub tx_hash: String,
  // pub rune_height: i32,
  // pub rune_index: i16,
  pub rune_id: String,
  #[diesel(deserialize_as = U128)]
  pub burned: u128,
  pub divisibility: i16,
  pub etching: String,
  pub mint_entry: MintEntryType,
  pub mints: i64,
  pub number: i64,
  pub rune: U128,
  pub spacers: i32,
  pub premine: i64,
  pub spaced_rune: String,
  pub supply: U128,
  pub symbol: Option<String>,
  pub timestamp: i32,
}

#[derive(Insertable, PartialEq, Debug, AsChangeset)]
#[diesel(table_name = crate::schema::transaction_rune_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTxRuneEntry<'a> {
  pub tx_hash: String,
  // pub rune_height: i32,
  // pub rune_index: i16,
  pub rune_id: String,
  #[diesel(serialize_as = U128)]
  pub burned: U128,
  pub divisibility: i16,
  pub etching: &'a str,
  pub mint_entry: MintEntryType,
  pub mints: i64,
  pub number: i64, //Block
  pub rune: U128,
  pub spacers: i32,
  pub premine: i64,
  pub spaced_rune: String,
  pub supply: U128,
  pub symbol: Option<&'a str>,
  pub timestamp: i32,
}

//TransactionRune
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::txid_runes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionRune {
  pub id: i64,
  pub tx_hash: String,
  pub rune_id: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::txid_runes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionRune {
  pub tx_hash: String,
  pub rune_id: String,
}

//TransactionRuneIdAddress
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::txid_rune_addresss)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TransactionRuneAddress {
  pub id: i64,
  pub tx_hash: String,
  pub rune_id: String,
  pub address: String,
  pub spent: bool,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::txid_rune_addresss)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewTransactionRuneAddress {
  pub tx_hash: String,
  pub rune_id: String,
  pub address: String,
  pub spent: bool,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::outpoint_rune_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OutpointRuneBalance {
  pub id: i64,
  pub tx_hash: String,
  pub vout: i32,
  pub rune_id: String,
  // pub rune_block: i32,
  // pub rune_tx: i16,
  #[diesel(serialize_as = U128, deserialize_as = U128)]
  pub balance_value: u128,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::outpoint_rune_balances)]
pub struct NewOutpointRuneBalance {
  pub tx_hash: String,
  pub vout: i32,
  pub rune_id: String,
  // pub rune_block: i32,
  // pub rune_tx: i16,
  pub balance_value: U128,
}
