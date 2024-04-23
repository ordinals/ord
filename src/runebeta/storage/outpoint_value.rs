use bitcoin::OutPoint;
use diesel::{
  query_dsl::methods::FilterDsl, ExpressionMethods, PgConnection, RunQueryDsl, SelectableHelper,
};

use crate::{runebeta::models::OutPointValue, schema::outpoint_values::dsl::*};

pub struct OutPointValueTable<'conn> {
  pub connection: &'conn mut PgConnection,
}

impl<'conn> OutPointValueTable<'conn> {
  pub fn new(connection: &'conn mut PgConnection) -> Self {
    Self { connection }
  }

  pub fn get(&self, outpoint: &OutPoint) -> Result<Option<i64>, diesel::result::Error> {
    let vout_value = outpoint.vout as i16;
    let result = outpoint_values
      .filter(tx_hash.eq(outpoint.txid.to_string()))
      .filter(vout.eq(&vout_value))
      .load::<OutPointValue>(self.connection)?
      .first()
      .map(|record| record.value);
    Ok(result)
    //.expect("Error saving satpoint")
  }
}
