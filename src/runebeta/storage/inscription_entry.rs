use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

use crate::{
  runebeta::models::{InscriptionEntry, NewInscriptionEntry},
  schema::inscription_entries::dsl::*,
};

pub struct InscriptionEntryTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> InscriptionEntryTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn get_last_sequence_number(&self) -> Result<i32, diesel::result::Error> {
    inscription_entries
      .select(sequence_number)
      .order_by(sequence_number.desc())
      .first(self.connection)
  }
}
