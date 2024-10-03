use {super::*, bitcoin::opcodes};

#[derive(Debug, Parser)]
pub struct Burn {
  #[arg(long, help = "Don't sign or broadcast transaction.")]
  dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB.")]
  fee_rate: FeeRate,
  #[arg(
    long,
    help = "Target <AMOUNT> postage with sent inscriptions. [default: 10000 sat]",
    value_name = "AMOUNT"
  )]
  postage: Option<Amount>,
  inscription: InscriptionId,
}

impl Burn {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let inscription_info = wallet
      .inscription_info()
      .get(&self.inscription)
      .ok_or_else(|| anyhow!("inscription {} not found", self.inscription))?
      .clone();

    let Some(value) = inscription_info.value else {
      bail!("Cannot burn unbound inscription");
    };

    ensure! {
      value <= TARGET_POSTAGE.to_sat(),
      "Cannot burn inscription contained in UTXO exceeding {TARGET_POSTAGE}",
    }

    ensure! {
      self.postage.unwrap_or_default() <= TARGET_POSTAGE,
      "Postage may not exceed {TARGET_POSTAGE}",
    }

    let unsigned_transaction = Self::create_unsigned_burn_transaction(
      &wallet,
      inscription_info.satpoint,
      self.postage,
      self.fee_rate,
    )?;

    let (txid, psbt, fee) = wallet.sign_transaction(unsigned_transaction, self.dry_run)?;

    Ok(Some(Box::new(send::Output {
      txid,
      psbt,
      outgoing: Outgoing::InscriptionId(self.inscription),
      fee,
    })))
  }

  fn create_unsigned_burn_transaction(
    wallet: &Wallet,
    satpoint: SatPoint,
    postage: Option<Amount>,
    fee_rate: FeeRate,
  ) -> Result<Transaction> {
    let runic_outputs = wallet.get_runic_outputs()?;

    ensure!(
      !runic_outputs.contains(&satpoint.outpoint),
      "runic outpoints may not be burned"
    );

    let change = [wallet.get_change_address()?, wallet.get_change_address()?];

    let postage = postage.map(Target::ExactPostage).unwrap_or(Target::Postage);

    let script_pubkey = script::Builder::new()
      .push_opcode(opcodes::all::OP_RETURN)
      .into_script();

    Ok(
      TransactionBuilder::new(
        satpoint,
        wallet.inscriptions().clone(),
        wallet.utxos().clone(),
        wallet.locked_utxos().clone().into_keys().collect(),
        runic_outputs,
        script_pubkey,
        change,
        fee_rate,
        postage,
        wallet.chain().network(),
      )
      .build_transaction()?,
    )
  }
}
