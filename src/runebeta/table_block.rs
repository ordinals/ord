use diesel::{associations::HasTable, PgConnection, RunQueryDsl, SelectableHelper};

use super::models::{Block, NewBlock};
use crate::schema::blocks::dsl::*;
#[derive(Clone)]
pub struct BlockTable {}

impl<'conn> BlockTable {
  pub fn new() -> Self {
    Self {}
  }
  pub fn insert(
    &self,
    block: &NewBlock,
    connection: &mut PgConnection,
  ) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(blocks::table())
      .values(block)
      .on_conflict(block_height)
      .do_update()
      .set(block)
      .returning(Block::as_returning())
      .execute(connection)
  }
}
