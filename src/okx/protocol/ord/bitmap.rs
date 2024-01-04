use crate::okx::datastore::ord::{OrdReader, OrdReaderWriter};
use crate::okx::protocol::context::Context;
use {
  crate::{
    okx::datastore::ord::{
      bitmap::District,
      collections::CollectionKind,
      operation::{Action, InscriptionOp},
    },
    Inscription, InscriptionId, Result,
  },
  anyhow::anyhow,
  bitcoin::Txid,
  std::collections::HashMap,
};

pub fn index_bitmap(
  context: &mut Context,
  operations: &HashMap<Txid, Vec<InscriptionOp>>,
) -> Result<u64> {
  let mut count = 0;

  // ignore transferred or cursed inscriptions.
  let mut positive_inscriptions = operations
    .values()
    .flatten()
    .cloned()
    .filter(|op| {
      !op.inscription_number.unwrap().is_negative() && matches!(op.action, Action::New { .. })
    })
    .collect::<Vec<_>>();

  // sort by inscription number.
  positive_inscriptions.sort_by_key(|op| op.inscription_number.unwrap());

  for op in positive_inscriptions.into_iter() {
    match op.action {
      Action::New {
        cursed: _,
        unbound: _,
        vindicated: _,
        inscription,
      } => {
        if let Some((inscription_id, district)) =
          index_district(context, inscription, op.inscription_id)?
        {
          let key = district.to_collection_key();
          context.set_inscription_by_collection_key(&key, &inscription_id)?;
          context.set_inscription_attributes(&inscription_id, &[CollectionKind::BitMap])?;

          count += 1;
        }
      }
      _ => unreachable!(),
    }
  }
  Ok(count)
}

fn index_district(
  context: &mut Context,
  inscription: Inscription,
  inscription_id: InscriptionId,
) -> Result<Option<(InscriptionId, District)>> {
  if let Some(content) = inscription.body() {
    if let Ok(district) = District::parse(content) {
      if district.number > context.chain.blockheight {
        return Ok(None);
      }
      let collection_key = district.to_collection_key();

      if context
        .get_collection_inscription_id(&collection_key)
        .map_err(|e| {
          anyhow!("failed to get collection inscription! key: {collection_key} error: {e}")
        })?
        .is_none()
      {
        log::info!(
          "found valid district! number: {} content: {} inscription_id {}",
          district.number,
          std::str::from_utf8(content).unwrap(),
          inscription_id,
        );
        return Ok(Some((inscription_id, district)));
      }
      log::info!(
        "duplicate district! number: {} content: {} inscription_id {}",
        district.number,
        std::str::from_utf8(content).unwrap(),
        inscription_id,
      );
    }
  }
  Ok(None)
}
