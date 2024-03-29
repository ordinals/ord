use diesel::{
  associations::HasTable, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};
use ordinals::SatPoint;

use crate::{
  runebeta::models::{NewSatpointEntity, SatpointEntity},
  schema::{block_headers::star, satpoints::dsl::*},
};

pub struct SatPointTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> SatPointTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn insert(
    &mut self,
    sequence: &i32,
    sat_point: &SatPoint,
  ) -> Result<SatpointEntity, diesel::result::Error> {
    let mut stored_entity = satpoints
      .filter(sequence_number.eq(sequence))
      .limit(1)
      .select(SatpointEntity::as_select())
      .load(self.connection)?
      .first();
    match stored_entity {
      Some(satpoint_entity) => {
        let vout_value = sat_point.outpoint.vout as i32;
        let offset = sat_point.offset as i64;
        diesel::update(satpoints.find(satpoint_entity.id))
          .set((
            tx_hash.eq(sat_point.outpoint.txid.to_string().as_str()),
            vout.eq::<&i32>(&vout_value),
            sat_offset.eq::<&i64>(&offset),
          ))
          .returning(SatpointEntity::as_returning())
          .get_result(self.connection)
      }
      None => {
        let payload = NewSatpointEntity {
          sequence_number: sequence.clone(),
          tx_hash: sat_point.outpoint.txid.to_string(),
          vout: sat_point.outpoint.vout.clone() as i32,
          sat_offset: sat_point.offset.clone() as i64,
        };
        Ok(
          diesel::insert_into(satpoints::table())
            .values(&payload)
            .returning(SatpointEntity::as_returning())
            .get_result(self.connection)
            .expect("Error saving satpoint"),
        )
      }
    }
  }
  pub fn create(
    &self,
    payload: &NewSatpointEntity,
  ) -> Result<SatpointEntity, diesel::result::Error> {
    diesel::insert_into(satpoints::table())
      .values(payload)
      .returning(SatpointEntity::as_returning())
      .get_result(self.connection)
    //.expect("Error saving satpoint")
  }
}
