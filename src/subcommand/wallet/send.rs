use {super::*, bitcoin::opcodes};

#[derive(Debug, Parser)]
pub(crate) struct Send {
  #[arg(long, help = "Don't sign or broadcast transaction")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
  #[arg(
    long,
    help = "Include OP_RETURN output with <DATA> in the transaction. Maximum 80 bytes.",
    value_name = "DATA"
  )]
  pub(crate) op_return: Option<String>,
  #[arg(
    long,
    help = "Target <AMOUNT> postage with sent inscriptions. [default: 10000 sat]",
    value_name = "AMOUNT"
  )]
  pub(crate) postage: Option<Amount>,
  #[arg(help = "Recipient address")]
  address: Address<NetworkUnchecked>,
  #[arg(
    help = "Outgoing asset formatted as a bitcoin amount, rune amount, sat name, satpoint, or \
    inscription ID. Bitcoin amounts are `DECIMAL UNIT` where `UNIT` is one of \
    `bit btc cbtc mbtc msat nbtc pbtc sat satoshi ubtc`. Rune amounts are `DECIMAL:RUNE` and \
    respect divisibility"
  )]
  asset: Outgoing,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub txid: Txid,
  pub psbt: String,
  pub asset: Outgoing,
  pub fee: u64,
}

impl Send {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let address = self
      .address
      .clone()
      .require_network(wallet.chain().network())?;

    // Validate OP_RETURN data if provided
    if let Some(ref data) = self.op_return {
      ensure!(
        data.len() <= 80,
        "OP_RETURN data exceeds maximum size of 80 bytes ({} bytes provided)",
        data.len()
      );
    }

    let mut unsigned_transaction = match self.asset {
      Outgoing::Amount(amount) => {
        wallet.create_unsigned_send_amount_transaction(address, amount, self.fee_rate)?
      }
      Outgoing::Rune { decimal, rune } => wallet.create_unsigned_send_or_burn_runes_transaction(
        Some(address),
        rune,
        decimal,
        self.postage,
        self.fee_rate,
      )?,
      Outgoing::InscriptionId(id) => wallet.create_unsigned_send_satpoint_transaction(
        address,
        wallet
          .inscription_info()
          .get(&id)
          .ok_or_else(|| anyhow!("inscription {id} not found"))?
          .satpoint,
        self.postage,
        self.fee_rate,
        true,
      )?,
      Outgoing::SatPoint(satpoint) => wallet.create_unsigned_send_satpoint_transaction(
        address,
        satpoint,
        self.postage,
        self.fee_rate,
        false,
      )?,
      Outgoing::Sat(sat) => wallet.create_unsigned_send_satpoint_transaction(
        address,
        wallet.find_sat_in_outputs(sat)?,
        self.postage,
        self.fee_rate,
        true,
      )?,
    };

    // Add OP_RETURN output if provided
    if let Some(data) = self.op_return.clone() {
      unsigned_transaction.output.push(TxOut {
        value: Amount::ZERO,
        script_pubkey: script::Builder::new()
          .push_opcode(opcodes::all::OP_RETURN)
          .push_slice(data.as_bytes())
          .into_script(),
      });
    }

    let (txid, psbt, fee) =
      wallet.sign_and_broadcast_transaction(unsigned_transaction, self.dry_run, None)?;

    Ok(Some(Box::new(Output {
      txid,
      psbt,
      asset: self.asset,
      fee,
    })))
  }
}
