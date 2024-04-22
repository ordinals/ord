use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Airdrop {
  #[clap(long, help = "Use <FEE_RATE> sats/vbyte for airdrop transaction.")]
  fee_rate: FeeRate,
  #[clap(long, help = "Airdrop <RUNE>. May contain `.` or `•`as spacers.")]
  rune: SpacedRune,
  #[clap(
    long,
    help = "Include <AMOUNT> postage with airdrop output. [default: 10000sat]"
  )]
  postage: Option<Amount>,
  #[clap(
    long,
    help = "Send minted runes to addresses listed in <DESTINATIONS> tsv file."
  )]
  destinations: PathBuf,
  #[arg(long, help = "Don't sign or broadcast transaction")]
  dry_run: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
  pub rune: SpacedRune,
  pub psbt: String,
  pub txid: Txid,
}

impl Airdrop {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "`ord wallet mint` requires index created with `--index-runes` flag",
    );

    let destinations = fs::read_to_string(self.destinations.clone())
      .with_context(|| format!("I/O error reading `{}`", self.destinations.display()))?
      .lines()
      .enumerate()
      .filter(|(_i, line)| !line.starts_with('#') && !line.is_empty())
      .filter_map(|(i, line)| {
        line.split('\t').next().map(|value| {
          Address::from_str(value)
            .map(|address| {
              address
                .require_network(wallet.chain().network())
                .map_err(|err| err.into())
            })
            .map_err(|err| {
              anyhow!(
                "failed to parse address from string \"{value}\" on line {}: {err}",
                i + 1,
              )
            })
        })
      })
      .flatten()
      .collect::<Result<Vec<Address>>>()?;

    let postage = self.postage.unwrap_or(TARGET_POSTAGE);

    ensure!(
      destinations
        .iter()
        .all(|destination| destination.script_pubkey().dust_value() <= postage),
      "postage below dust limit",
    );

    let rune = self.rune.rune;

    let Some((id, rune_entry, _)) = wallet.get_rune(rune)? else {
      bail!("rune {rune} has not been etched");
    };

    let inscribed_outputs = wallet
      .inscriptions()
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<HashSet<OutPoint>>();

    wallet.lock_non_cardinal_outputs()?;

    let mut inputs = Vec::new();
    for output in wallet.get_runic_outputs()? {
      if inscribed_outputs.contains(&output) {
        continue;
      }

      let runes = wallet.get_runes_balances_for_output(&output)?;

      // makes sure input contains only one type of rune
      if runes
        .iter()
        .all(|(spaced_rune, _)| rune == spaced_rune.rune)
      {
        inputs.push(output);
      }
    }

    if inputs.is_empty() {
      return Err(anyhow!("rune {rune} not in wallet"));
    }

    let runestone = Runestone {
      edicts: vec![Edict {
        amount: 0,
        id,
        output: destinations.len() as u32 + 1,
      }],
      ..default()
    };

    ensure!(
      runestone.encipher().len() <= 82,
      "runestone greater than maximum OP_RETURN size: {} > 82",
      runestone.encipher().len()
    );

    let mut output = destinations
      .into_iter()
      .map(|destination| TxOut {
        script_pubkey: destination.script_pubkey(),
        value: postage.to_sat(),
      })
      .collect::<Vec<TxOut>>();

    output.insert(
      0,
      TxOut {
        script_pubkey: runestone.encipher(),
        value: 0,
      },
    );

    let unfunded_transaction = Transaction {
      version: 2,
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
      output,
    };

    let unsigned_transaction = fund_raw_transaction(
      wallet.bitcoin_client(),
      self.fee_rate,
      &unfunded_transaction,
    )?;

    let unsigned_transaction = consensus::encode::deserialize(&unsigned_transaction)?;

    assert_eq!(
      Runestone::decipher(&unsigned_transaction),
      Some(Artifact::Runestone(runestone)),
    );

    let unspent_outputs = wallet.utxos();

    let (txid, psbt) = if self.dry_run {
      let psbt = wallet
        .bitcoin_client()
        .wallet_process_psbt(
          &base64::engine::general_purpose::STANDARD
            .encode(Psbt::from_unsigned_tx(unsigned_transaction.clone())?.serialize()),
          Some(false),
          None,
          None,
        )?
        .psbt;

      (unsigned_transaction.txid(), psbt)
    } else {
      let psbt = wallet
        .bitcoin_client()
        .wallet_process_psbt(
          &base64::engine::general_purpose::STANDARD
            .encode(Psbt::from_unsigned_tx(unsigned_transaction.clone())?.serialize()),
          Some(true),
          None,
          None,
        )?
        .psbt;

      let signed_tx = wallet
        .bitcoin_client()
        .finalize_psbt(&psbt, None)?
        .hex
        .ok_or_else(|| anyhow!("unable to sign transaction"))?;

      (
        wallet.bitcoin_client().send_raw_transaction(&signed_tx)?,
        psbt,
      )
    };

    let mut fee = 0;
    for txin in unsigned_transaction.input.iter() {
      let Some(txout) = unspent_outputs.get(&txin.previous_output) else {
        panic!("input {} not found in utxos", txin.previous_output);
      };
      fee += txout.value;
    }

    for txout in unsigned_transaction.output.iter() {
      fee = fee.checked_sub(txout.value).unwrap();
    }

    Ok(Some(Box::new(Output {
      rune: self.rune,
      psbt,
      txid,
    })))
  }
}
