use {super::*, crate::outgoing::Outgoing};

pub(crate) struct AmountTransfer {
  pub(crate) amount: Amount,
}
impl Transfer for AmountTransfer {
  fn get_outgoing(&self) -> Outgoing {
    Outgoing::Amount(self.amount)
  }

  fn create_unsigned_transaction(
    &self,
    wallet: &Wallet,
    destination: Address,
    _postage: Option<Amount>,
    fee_rate: FeeRate,
  ) -> crate::Result<Transaction> {
    wallet.lock_non_cardinal_outputs()?;

    let unfunded_transaction = Transaction {
      version: 2,
      lock_time: LockTime::ZERO,
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: destination.script_pubkey(),
        value: self.amount.to_sat(),
      }],
    };

    let unsigned_transaction = consensus::encode::deserialize(&fund_raw_transaction(
      wallet.bitcoin_client(),
      fee_rate,
      &unfunded_transaction,
    )?)?;

    Ok(unsigned_transaction)
  }
}
