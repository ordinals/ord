use super::*;

pub(crate) fn run(options: Options) -> Result {
  for utxo in get_wallet(options)?.list_unspent()? {
    println!(
      "{}:{} {}",
      utxo.outpoint.txid, utxo.outpoint.vout, utxo.txout.value
    );
  }
  Ok(())
}
