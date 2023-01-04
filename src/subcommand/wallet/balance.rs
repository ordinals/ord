use super::*;

pub(crate) fn run(options: Options) -> Result {
  println!(
    "{}",
    options
      .bitcoin_rpc_client(0)?
      .get_balances()?
      .mine
      .trusted
      .to_sat()
  );

  Ok(())
}
