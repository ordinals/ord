use super::*;

pub(crate) fn run(options: Options) -> Result {
  let outputs = options
    .bitcoin_rpc_client_for_wallet_command("ord wallet outputs")?
    .list_unspent(None, None, None, None, None)?
    .iter()
    .map(|output| (OutPoint::new(output.txid, output.vout), output.amount))
    .collect::<Vec<(OutPoint, Amount)>>();

  for (outpoint, amount) in outputs {
    println!("{outpoint}\t{}", amount.to_sat());
  }

  Ok(())
}
