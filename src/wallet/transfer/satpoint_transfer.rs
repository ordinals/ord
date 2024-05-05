use {crate::outgoing::Outgoing, super::*};

pub(crate) struct SatPointTransfer {
    pub(crate) satpoint: SatPoint
}

impl Transfer for SatPointTransfer {
    fn get_outgoing(&self) -> Outgoing {
        Outgoing::SatPoint(self.satpoint)
    }

    fn create_unsigned_transaction(&self, wallet: &Wallet, destination: Address, postage: Option<Amount>, fee_rate: FeeRate) -> crate::Result<Transaction> {
        self.create_unsigned_send_satpoint_transaction(wallet, destination, self.satpoint, postage, fee_rate)
    }
}