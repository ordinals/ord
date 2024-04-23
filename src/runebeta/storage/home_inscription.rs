use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

use crate::{
  runebeta::models::{Inscriptions, NewInscriptions},
  schema::inscriptions::dsl::*,
};

pub struct HomeInscriptionTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> HomeInscriptionTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn count(&self) -> Result<i64, diesel::result::Error> {
    inscriptions.count().get_result(self.connection)
  }
}
