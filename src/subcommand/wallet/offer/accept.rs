use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub txid: Txid,
}

#[derive(Debug, Parser)]
pub(crate) struct Accept {
  #[arg(long, help = "Assert offer is for <AMOUNT>")]
  amount: Amount,
  #[arg(long, help = "Don't sign or broadcast transaction")]
  dry_run: bool,
  #[arg(long, help = "Assert offer is for <INSCRIPTION>")]
  inscription: InscriptionId,
  #[arg(long, help = "Accept <PSBT> offer")]
  psbt: String,
}

impl Accept {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let accept = buy_offer::accept::Accept {
      outgoing: vec![Outgoing::InscriptionId(self.inscription)],
      amount: self.amount,
      dry_run: self.dry_run,
      psbt: self.psbt.clone(),
    };

    let (txid, _, _) = accept.accept_inscription_buy_offer(wallet, self.inscription)?;

    Ok(Some(Box::new(Output { txid })))
  }
}
