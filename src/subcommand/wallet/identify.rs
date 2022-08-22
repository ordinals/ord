use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::index(&options)?;

  let utxos = Purse::load(&options)?.wallet.list_unspent()?;

  let ranges = utxos
    .iter()
    .map(|utxo| index.list(utxo.outpoint))
    .collect::<Result<Vec<Option<List>>, _>>()?;

  for (utxo, range) in utxos.iter().zip(ranges.into_iter()) {
    match range {
      Some(List::Unspent(range)) => {
        for (start, _) in range {
          let ordinal = Ordinal(start);

          let rarity = ordinal.rarity();

          if rarity != "common" {
            println!("{ordinal} {rarity} {}", utxo.outpoint);
          }
        }
      }
      Some(List::Spent(txid)) => {
        return Err(anyhow!(
          "UTXO {} unspent in wallet but spent in index by transaction {txid}",
          utxo.outpoint
        ))
      }
      None => {}
    }
  }

  Ok(())
}
