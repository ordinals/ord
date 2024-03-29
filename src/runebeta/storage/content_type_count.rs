use diesel::{PgConnection, QueryDsl, RunQueryDsl};

use crate::{
  runebeta::models::{ContentTypeCount, NewContentTypeCount},
  schema::content_type_counts::dsl::*,
};

pub struct ContentTypeCountTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> ContentTypeCountTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  // pub fn count(&self) -> Result<ContentTypeCount, diesel::result::Error> {
  //   inscriptions.count().get_result(self.connection)
  // }
}
