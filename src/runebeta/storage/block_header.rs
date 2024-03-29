use bitcoin::{block::Header, Block};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::{
  runebeta::models::{BlockHeader, NewBlockHeader},
  schema::block_headers::dsl::*,
};

pub struct BlockHeaderTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> BlockHeaderTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn insert(&self, payload: &NewBlockHeader) -> Result<BlockHeader, diesel::result::Error> {
    diesel::insert_into(crate::schema::block_headers::table)
      .values(payload)
      .returning(BlockHeader::as_returning())
      .get_result(self.connection)
    //.expect("Error saving satpoint")
  }
  pub fn get_indexed_block(&mut self) -> Result<i64, diesel::result::Error> {
    block_headers
      .order(height.desc())
      .select(height)
      .first::<i64>(self.connection)
      .map(|value| value + 1)
  }

  //   pub fn set_block_header(
  //     &mut self,
  //     heigh: i32,
  //     header: &Header,
  //   ) -> Result<BlockHeader, diesel::result::Error> {
  //   }
}
