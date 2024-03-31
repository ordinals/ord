use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
  pub rune: Rune,
  pub reveal: Txid,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let mut output: Option<Box<dyn subcommand::Output>> = None;
  for (rune, entry) in wallet.pending()? {
    output = Some(Box::new(wallet.wait_for_maturation(
      &rune,
      entry.commit,
      entry.reveal,
      entry.output.clone(),
    )?) as Box<dyn subcommand::Output>);
  }

  Ok(output)
}
