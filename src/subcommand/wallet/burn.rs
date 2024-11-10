use {super::*, bitcoin::opcodes};

#[derive(Debug, Parser)]
pub struct Burn {
  #[arg(
    long,
    conflicts_with = "json_metadata",
    help = "Include CBOR from <PATH> in OP_RETURN.",
    value_name = "PATH"
  )]
  cbor_metadata: Option<PathBuf>,
  #[arg(long, help = "Don't sign or broadcast transaction.")]
  dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB.")]
  fee_rate: FeeRate,
  #[arg(
    long,
    help = "Include JSON from <PATH> converted to CBOR in OP_RETURN.",
    conflicts_with = "cbor_metadata",
    value_name = "PATH"
  )]
  json_metadata: Option<PathBuf>,
  #[arg(
    long,
    alias = "nolimit",
    help = "Allow OP_RETURN greater than 83 bytes. Transactions over this limit are nonstandard \
    and will not be relayed by bitcoind in its default configuration. Do not use this flag unless \
    you understand the implications."
  )]
  no_limit: bool,
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

    let metadata = WalletCommand::parse_metadata(self.cbor_metadata, self.json_metadata)?;

    let Some(value) = inscription_info.value else {
      bail!("Cannot burn unbound inscription");
    };

    let value = Amount::from_sat(value);

    ensure! {
      value <= TARGET_POSTAGE,
      "Cannot burn inscription contained in UTXO exceeding {TARGET_POSTAGE}",
    }

    ensure! {
      self.postage.unwrap_or_default() <= TARGET_POSTAGE,
      "Postage may not exceed {TARGET_POSTAGE}",
    }

    let mut builder = script::Builder::new().push_opcode(opcodes::all::OP_RETURN);

    if let Some(metadata) = metadata {
      let push: &script::PushBytes = metadata.as_slice().try_into().with_context(|| {
        format!(
          "metadata length {} over maximum {}",
          metadata.len(),
          u32::MAX
        )
      })?;
      builder = builder.push_slice(push);
    }

    let script_pubkey = builder.into_script();

    ensure!(
      self.no_limit || script_pubkey.len() <= MAX_STANDARD_OP_RETURN_SIZE,
      "OP_RETURN with metadata larger than maximum: {} > {}",
      script_pubkey.len(),
      MAX_STANDARD_OP_RETURN_SIZE,
    );

    let unsigned_transaction = Self::create_unsigned_burn_transaction(
      &wallet,
      inscription_info.satpoint,
      self.postage,
      self.fee_rate,
      script_pubkey,
    )?;

    let (txid, psbt, fee) =
      wallet.sign_and_broadcast_transaction(unsigned_transaction, self.dry_run)?;

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
    script_pubkey: ScriptBuf,
  ) -> Result<Transaction> {
    let runic_outputs = wallet.get_runic_outputs()?;

    ensure!(
      !runic_outputs.contains(&satpoint.outpoint),
      "runic outpoints may not be burned"
    );

    let change = [wallet.get_change_address()?, wallet.get_change_address()?];

    let postage = postage.map(Target::ExactPostage).unwrap_or(Target::Postage);

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
