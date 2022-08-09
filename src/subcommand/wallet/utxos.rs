use super::*;

pub(crate) fn run(options: Options) -> Result {
  println!(
    "{}",
    get_wallet(options)?
      .list_unspent()?
      .iter()
      .map(|utxo| format!("{}:{}", utxo.outpoint.txid, utxo.outpoint.vout))
      .collect::<Vec<String>>()
      .join("\n")
  );
  Ok(())
}
