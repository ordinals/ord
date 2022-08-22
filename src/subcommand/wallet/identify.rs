use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::index(&options)?;

  let ranges = Purse::load(&options)?
    .wallet
    .list_unspent()?
    .iter()
    .map(|utxo| (utxo.clone(), index.list(utxo.outpoint).unwrap()))
    .collect::<Vec<(LocalUtxo, Option<List>)>>();

  for (utxo, range) in ranges {
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
