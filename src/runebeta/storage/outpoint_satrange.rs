use bitcoin::OutPoint;
use diesel::{
  associations::HasTable, query_dsl::methods::FilterDsl, ExpressionMethods, PgConnection,
  RunQueryDsl, SelectableHelper,
};

use crate::{
  runebeta::models::{NewOutpointSatRange, OutpointSatRange},
  schema::outpoint_satranges::dsl::*,
};

pub struct OutPointSatrangeTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> OutPointSatrangeTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }
  pub fn get(&self, outpoint: &OutPoint) -> Result<Option<Vec<u8>>, diesel::result::Error> {
    let vout_value = outpoint.vout as i16;
    let res = outpoint_satranges
      .filter(tx_hash.eq(outpoint.txid.to_string()))
      .filter(vout.eq(&vout_value))
      .load::<OutpointSatRange>(self.connection)?
      .first()
      .map(|record| record.range);
    Ok(res)
    //.expect("Error saving satpoint")
  }

  pub fn remove(&self, outpoint: &OutPoint) -> Result<usize, diesel::result::Error> {
    let vout_value = outpoint.vout as i16;
    diesel::delete(
      outpoint_satranges
        .filter(tx_hash.eq(outpoint.txid.to_string()))
        .filter(vout.eq::<i16>(vout_value)),
    )
    .execute(self.connection)
    //.expect("Error saving satpoint")
  }
  pub fn create(
    &self,
    outpoint: &OutPoint,
    sat_range: &Vec<u8>,
  ) -> Result<OutpointSatRange, diesel::result::Error> {
    let new_outpoint_sat_range = NewOutpointSatRange {
      tx_hash: outpoint.txid.to_string().as_str(),
      vout: outpoint.vout as i16,
      range: sat_range,
    };
    diesel::insert_into(outpoint_satranges::table())
      .values(&new_outpoint_sat_range)
      .returning(OutpointSatRange::as_returning())
      .get_result(self.connection)
  }
  pub fn upsert_values(&self, payload: &NewOutpointSatRange) -> Result<(), diesel::result::Error> {
    diesel::insert_into(outpoint_satranges::table())
      .values(payload)
      .on_conflict((tx_hash, vout))
      .do_update()
      .set(payload)
      .returning(OutpointSatRange::as_returning())
      .execute(self.connection);
    //.expect("Error saving satpoint")
    Ok(())
  }
}
