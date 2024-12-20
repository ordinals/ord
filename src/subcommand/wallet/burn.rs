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
  asset: Outgoing,
}

impl Burn {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let (unsigned_transaction, burn_amount) = match self.asset {
      Outgoing::InscriptionId(id) => {
        let inscription_info = wallet
          .inscription_info()
          .get(&id)
          .ok_or_else(|| anyhow!("inscription {id} not found"))?
          .clone();

        let metadata = WalletCommand::parse_metadata(self.cbor_metadata, self.json_metadata)?;

        ensure!(
          inscription_info.value.is_some(),
          "Cannot burn unbound inscription"
        );

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

        let burn_amount = Amount::from_sat(1);

        (
          Self::create_unsigned_burn_satpoint_transaction(
            &wallet,
            inscription_info.satpoint,
            self.fee_rate,
            script_pubkey,
            burn_amount,
          )?,
          burn_amount,
        )
      }
      Outgoing::Rune { decimal, rune } => {
        ensure!(
          self.cbor_metadata.is_none() && self.json_metadata.is_none(),
          "metadata not supported when burning runes"
        );

        (
          wallet.create_unsigned_send_or_burn_runes_transaction(
            None,
            rune,
            decimal,
            None,
            self.fee_rate,
          )?,
          Amount::ZERO,
        )
      }
      Outgoing::Amount(_) => bail!("burning bitcoin not supported"),
      Outgoing::Sat(_) => bail!("burning sat not supported"),
      Outgoing::SatPoint(_) => bail!("burning satpoint not supported"),
    };

    let base_size = unsigned_transaction.base_size();

    assert!(
      base_size >= 65,
      "transaction base size less than minimum standard tx nonwitness size: {base_size} < 65",
    );

    let (txid, psbt, fee) = wallet.sign_and_broadcast_transaction(
      unsigned_transaction,
      self.dry_run,
      Some(burn_amount),
    )?;

    Ok(Some(Box::new(send::Output {
      txid,
      psbt,
      asset: self.asset,
      fee,
    })))
  }

  fn create_unsigned_burn_satpoint_transaction(
    wallet: &Wallet,
    satpoint: SatPoint,
    fee_rate: FeeRate,
    script_pubkey: ScriptBuf,
    burn_amount: Amount,
  ) -> Result<Transaction> {
    let runic_outputs = wallet.get_runic_outputs()?.unwrap_or_default();

    ensure!(
      !runic_outputs.contains(&satpoint.outpoint),
      "runic outpoints may not be burned"
    );

    let change = [wallet.get_change_address()?, wallet.get_change_address()?];

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
        Target::ExactPostage(burn_amount),
        wallet.chain().network(),
      )
      .build_transaction()?,
    )
  }
}
