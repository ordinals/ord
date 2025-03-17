use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub psbt: String,
  pub seller_address: Address<NetworkUnchecked>,
  pub inscription: InscriptionId,
}

#[derive(Debug, Parser)]
pub(crate) struct Create {
  #[arg(long, help = "<INSCRIPTION> to make offer for.")]
  inscription: InscriptionId,
  #[arg(long, help = "<AMOUNT> to offer.")]
  amount: Amount,
  #[arg(long, help = "<FEE_RATE> for finalized transaction.")]
  fee_rate: FeeRate,
}

impl Create {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let create = buy_offer::create::Create {
      outgoing: Outgoing::InscriptionId(self.inscription),
      amount: self.amount,
      fee_rate: self.fee_rate,
      utxo: None,
      postage: None,
    };

    let (psbt, seller_address) = create.create_inscription_buy_offer(wallet, self.inscription)?;

    Ok(Some(Box::new(Output {
      psbt,
      seller_address,
      inscription: self.inscription,
    })))
  }
}
