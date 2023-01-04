use super::*;

pub(crate) fn run(options: Options) -> Result {
  println!(
    "{}",
    options
      .bitcoin_rpc_client_for_wallet_command("ord wallet receive", 0)?
      .get_new_address(None, None)?
  );

  Ok(())
}
