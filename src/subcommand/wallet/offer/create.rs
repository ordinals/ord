use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Create {
  #[arg(short, long, help = "<INSCRIPTION> to make offer for.")]
  inscription: InscriptionId,
}

impl Create {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    todo!()
  }
}
