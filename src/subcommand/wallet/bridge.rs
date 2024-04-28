use {super::*, crate::bridge::lock, crate::outgoing::Outgoing};

#[derive(Debug, Parser)]
pub(crate) struct Lock {
  outgoing: Outgoing,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
}

impl Lock {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "bridging runes with `ord wallet bridge-lock` requires index created with `--index-runes` flag",
    );

    let output = lock(wallet, self.fee_rate, self.outgoing)?;

    Ok(Some(Box::new(output)))
  }
}

#[derive(Debug, Parser)]
pub(crate) struct Unlock {}

impl Unlock {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "bridging runes with `ord wallet bridge-unlock` requires index created with `--index-runes` flag",
    );

    println!("unlock");

    Ok(None)
  }
}
