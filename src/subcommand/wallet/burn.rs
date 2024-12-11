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
          "metadata not supported for burning runes"
        );

        (
          Self::create_unsigned_burn_runes_transaction(&wallet, rune, decimal, self.fee_rate)?,
          Amount::ZERO,
        )
      }
      _ => bail!("burning is only implemented for inscriptions and runes"),
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
    let runic_outputs = wallet.get_runic_outputs()?;

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

  fn create_unsigned_burn_runes_transaction(
    wallet: &Wallet,
    spaced_rune: SpacedRune,
    decimal: Decimal,
    fee_rate: FeeRate,
  ) -> Result<Transaction> {
    ensure!(
      wallet.has_rune_index(),
      "sending runes with `ord send` requires index created with `--index-runes` flag",
    );

    wallet.lock_non_cardinal_outputs()?;

    let (id, entry, _parent) = wallet
      .get_rune(spaced_rune.rune)?
      .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;

    let amount = decimal.to_integer(entry.divisibility)?;

    let inscribed_outputs = wallet
      .inscriptions()
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<HashSet<OutPoint>>();

    let balances = wallet
      .get_runic_outputs()?
      .into_iter()
      .filter(|output| !inscribed_outputs.contains(output))
      .map(|output| {
        wallet.get_runes_balances_in_output(&output).map(|balance| {
          (
            output,
            balance
              .into_iter()
              .map(|(spaced_rune, pile)| (spaced_rune.rune, pile.amount))
              .collect(),
          )
        })
      })
      .collect::<Result<BTreeMap<OutPoint, BTreeMap<Rune, u128>>>>()?;

    let mut inputs = Vec::new();
    let mut input_rune_balances: BTreeMap<Rune, u128> = BTreeMap::new();

    for (output, runes) in balances {
      if let Some(balance) = runes.get(&spaced_rune.rune) {
        if *balance > 0 {
          for (rune, balance) in runes {
            *input_rune_balances.entry(rune).or_default() += balance;
          }

          inputs.push(output);

          if input_rune_balances
            .get(&spaced_rune.rune)
            .cloned()
            .unwrap_or_default()
            >= amount
          {
            break;
          }
        }
      }
    }

    let input_rune_balance = input_rune_balances
      .get(&spaced_rune.rune)
      .cloned()
      .unwrap_or_default();

    let needs_runes_change_output = input_rune_balance > amount || input_rune_balances.len() > 1;

    ensure! {
      input_rune_balance >= amount,
      "insufficient `{}` balance, only {} in wallet",
      spaced_rune,
      Pile {
        amount: input_rune_balance,
        divisibility: entry.divisibility,
        symbol: entry.symbol
      },
    }

    let runestone = Runestone {
      edicts: vec![Edict {
        amount,
        id,
        output: 0,
      }],
      ..default()
    };

    let unfunded_transaction = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: inputs
        .into_iter()
        .map(|previous_output| TxIn {
          previous_output,
          script_sig: ScriptBuf::new(),
          sequence: Sequence::MAX,
          witness: Witness::new(),
        })
        .collect(),
      output: if needs_runes_change_output {
        vec![
          TxOut {
            script_pubkey: runestone.encipher(),
            value: Amount::from_sat(0),
          },
          TxOut {
            script_pubkey: wallet.get_change_address()?.script_pubkey(),
            value: TARGET_POSTAGE,
          },
        ]
      } else {
        vec![TxOut {
          script_pubkey: runestone.encipher(),
          value: Amount::from_sat(0),
        }]
      },
    };

    let unsigned_transaction =
      fund_raw_transaction(wallet.bitcoin_client(), fee_rate, &unfunded_transaction)?;

    let unsigned_transaction = consensus::encode::deserialize(&unsigned_transaction)?;

    if needs_runes_change_output {
      assert_eq!(
        Runestone::decipher(&unsigned_transaction),
        Some(Artifact::Runestone(runestone)),
      );
    }

    Ok(unsigned_transaction)
  }
}
