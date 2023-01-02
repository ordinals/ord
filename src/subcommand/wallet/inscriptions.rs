use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.update()?;

  let inscriptions = index.get_inscriptions(None)?;
  let utxos = list_utxos(&options)?;

  for (satpoint, inscription_id) in inscriptions {
    if utxos.contains_key(&satpoint.outpoint) {
      println!("{}\t{}", inscription_id, satpoint);
    }
  }

  Ok(())
}
