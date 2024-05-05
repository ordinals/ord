use {super::*, crate::outgoing::Outgoing};

pub(crate) struct SatTransfer {
  pub(crate) sat: Sat,
}

impl Transfer for SatTransfer {
  fn get_outgoing(&self) -> Outgoing {
    Outgoing::Sat(self.sat)
  }
  fn create_unsigned_transaction(
    &self,
    wallet: &Wallet,
    destination: Address,
    postage: Option<Amount>,
    fee_rate: FeeRate,
  ) -> crate::Result<Transaction> {
    let satpoint = wallet.find_sat_in_outputs(self.sat)?;
    self.create_unsigned_send_satpoint_transaction(wallet, destination, satpoint, postage, fee_rate)
  }
}
