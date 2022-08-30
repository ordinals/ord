use super::*;

pub(crate) fn run(options: Options) -> Result {
  let mut utxos = Purse::load(&options)?.wallet.list_unspent()?;

  utxos.sort_by_key(|utxo| utxo.outpoint);

  for utxo in utxos {
    println!(
      "{}:{} {}",
      utxo.outpoint.txid, utxo.outpoint.vout, utxo.txout.value
    );
  }

  Ok(())
}
