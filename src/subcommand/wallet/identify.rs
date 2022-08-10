use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::index(&options)?;

  let ranges = get_wallet(options)?
    .list_unspent()?
    .iter()
    .map(|utxo| index.list(utxo.outpoint))
    .collect::<Result<Vec<Option<Vec<(u64, u64)>>>, _>>()?;

  for range in ranges.into_iter().flatten() {
    for (start, end) in range {
      println!("[{}, {})", start, end);
    }
  }

  Ok(())
}
