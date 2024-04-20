use super::*;
use crate::outgoing::Outgoing;

#[derive(Debug, Parser)]
pub(crate) struct Lock {
  outgoing: Outgoing,
}

// create_unsigned_send_runes_transaction

impl Lock {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "bridging runes with `ord wallet bridge-lock` requires index created with `--index-runes` flag",
    );

    match self.outgoing {
      Outgoing::Rune {
        decimal,
        rune: spaced_rune,
      } => {
        let (id, entry, _parent) = wallet
          .get_rune(spaced_rune.rune)?
          .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;

        let amount = decimal.to_integer(entry.divisibility)?;

        println!("rune: id {}, amount {}", id, amount);
      }
      _ => todo!("currently only runes can be bridged"),
    }

    Ok(None)
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
