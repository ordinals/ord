use diesel::{
  associations::HasTable, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};

use crate::{
  runebeta::models::{HeightSequenceNumber, NewHeightSequenceNumber},
  schema::height_sequence_numbers::dsl::*,
};

pub struct HeightSequenceNumberTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> HeightSequenceNumberTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn insert(
    &self,
    height_value: i32,
    sequence_number_value: i32,
  ) -> Result<HeightSequenceNumber, diesel::result::Error> {
    match height_sequence_numbers
      .filter(height.eq(height_value))
      .limit(1)
      .load::<HeightSequenceNumber>(self.connection)?
      .first()
    {
      Some(record) => diesel::update(height_sequence_numbers.find(record.id))
        .set(sequence_number.eq(sequence_number_value))
        .returning(HeightSequenceNumber::as_returning())
        .get_result(self.connection),
      None => {
        let payload = NewHeightSequenceNumber {
          height: height_value,
          sequence_number: sequence_number_value,
        };
        diesel::insert_into(height_sequence_numbers::table())
          .values(payload)
          .returning(HeightSequenceNumber::as_returning())
          .get_result(self.connection)
        //.expect("Error saving satpoint")
      }
    }
  }
}
