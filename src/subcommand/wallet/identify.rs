use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::index(&options)?;

  let ranges = Purse::load(&options)?
    .wallet
    .list_unspent()?
    .iter()
    .map(|utxo| index.list(utxo.outpoint))
    .collect::<Result<Vec<Option<List>>, _>>()?;

  for range in ranges.into_iter().flatten() {
    match range {
      List::Unspent(range) => {
        for (start, end) in range {
          println!("[{}, {})", start, end);
        }
      }
      List::Spent(txid) => {
        return Err(anyhow!(
          "UTXO unspent in wallet but spent in index by transaction {txid}"
        ))
      }
    }
  }

  Ok(())
}
