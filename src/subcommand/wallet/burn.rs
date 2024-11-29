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

    let mut builder = script::Builder::new().push_opcode(opcodes::all::OP_RETURN);

    // add empty metadata if none is supplied so we can add padding
    let metadata = metadata.unwrap_or_default();

    let push: &script::PushBytes = metadata.as_slice().try_into().with_context(|| {
      format!(
        "metadata length {} over maximum {}",
        metadata.len(),
        u32::MAX
      )
    })?;
    builder = builder.push_slice(push);

    // pad OP_RETURN script to least five bytes to ensure transaction base size
    // is greater than 64 bytes
    let padding = 5usize.saturating_sub(builder.as_script().len());
    if padding > 0 {
      // subtract one byte push opcode from padding length
      let padding = vec![0; padding - 1];
      let push: &script::PushBytes = padding.as_slice().try_into().unwrap();
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
      self.fee_rate,
      script_pubkey,
    )?;

    let base_size = unsigned_transaction.base_size();
    assert!(
      base_size >= 65,
      "transaction base size less than minimum standard tx nonwitness size: {base_size} < 65",
    );

    let (txid, psbt, fee) = wallet.sign_and_broadcast_transaction(
      unsigned_transaction,
      self.dry_run,
      Some(Amount::from_sat(value)),
    )?;

    Ok(Some(Box::new(send::Output {
      txid,
      psbt,
      asset: Outgoing::InscriptionId(self.inscription),
      fee,
    })))
  }

  fn create_unsigned_burn_transaction(
    wallet: &Wallet,
    satpoint: SatPoint,
    fee_rate: FeeRate,
    script_pubkey: ScriptBuf,
  ) -> Result<Transaction> {
    let runic_outputs = wallet.get_runic_outputs()?;

    ensure!(
      !runic_outputs.contains(&satpoint.outpoint),
      "runic outpoints may not be burned"
    );

    let change = [wallet.get_change_address()?, wallet.get_change_address()?];
    let postage = Target::ExactPostage(Amount::from_sat(1));

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
