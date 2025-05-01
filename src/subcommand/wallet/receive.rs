use super::*;

#[derive(Deserialize, Serialize)]
pub struct Output {
  pub addresses: Vec<Address<NetworkUnchecked>>,
}

#[derive(Debug, Parser)]
pub(crate) struct Receive {
  #[arg(
    short,
    long,
    help = "Generate <NUMBER> addresses.",
    default_value_t = 1
  )]
  number: usize,
}

impl Receive {
  pub(crate) fn run(self, mut wallet: Wallet) -> SubcommandResult {
    let addresses = wallet
      .get_receive_addresses(self.number)
      .into_iter()
      .map(|address| address.into_unchecked())
      .collect();

    wallet.persist()?;

    Ok(Some(Box::new(Output { addresses })))
  }
}
