use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Freeze {
  #[clap(long, help = "Use <FEE_RATE> sats/vbyte for freeze transaction.")]
  fee_rate: FeeRate,
  #[clap(long, help = "Freeze <RUNE>. May contain `.` or `•` as spacers.")]
  rune: SpacedRune,
  #[clap(long, help = "Freeze runes at <OUTPOINTS>.")]
  outpoints: Vec<OutPoint>,
  #[clap(
    long,
    help = "Include <AMOUNT> postage with mint output. [default: 10000sat]"
  )]
  postage: Option<Amount>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
  pub rune: SpacedRune,
  pub freezer: SpacedRune,
  pub txid: Txid,
}

impl Freeze {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "`ord wallet freeze` requires index created with `--index-runes` flag",
    );

    let rune = self.rune.rune;

    let bitcoin_client = wallet.bitcoin_client();

    let Some((id, rune_entry, _)) = wallet.get_rune(rune)? else {
      bail!("rune {rune} has not been etched");
    };

    let Some(freezer) = rune_entry.freezer else {
      bail!("rune {rune} not freezable");
    };

    let Some((_, freezer_entry, _)) = wallet.get_rune(freezer)? else {
      bail!("freezer rune {freezer} has not been etched");
    };

    let balances = wallet
      .get_runic_outputs()?
      .unwrap_or_default()
      .into_iter()
      .map(|output| {
        wallet.get_runes_balances_in_output(&output).map(|balance| {
          (
            output,
            balance
              .unwrap_or_default()
              .into_iter()
              .map(|(spaced_rune, pile)| (spaced_rune.rune, pile.amount))
              .collect(),
          )
        })
      })
      .collect::<Result<BTreeMap<OutPoint, BTreeMap<Rune, u128>>>>()?;

    let mut input = None;
    for (output, runes) in balances {
      if let Some(balance) = runes.get(&freezer) {
        if *balance > 0 {
          input = Some(output);
          break;
        }
      }
    }

    let Some(input) = input else {
      bail!(
        "insufficient `{}` balance, 0 in wallet",
        freezer_entry.spaced_rune
      );
    };

    let postage = self.postage.unwrap_or(TARGET_POSTAGE);

    let destination = wallet.get_change_address()?;

    ensure!(
      destination.script_pubkey().minimal_non_dust() <= postage,
      "postage below dust limit of {}sat",
      destination.script_pubkey().minimal_non_dust().to_sat()
    );

    let mut outpoints = Vec::new();
    for outpoint in self.outpoints {
      let tx_info = bitcoin_client.get_transaction(&outpoint.txid, None)?.info;
      let Some(blockheight) = tx_info.blockheight else {
        bail!("outpoint `{}` not found", outpoint);
      };
      let Some(blockindex) = tx_info.blockindex else {
        bail!("outpoint `{}` not found", outpoint);
      };
      outpoints.push(
        OutPointId::new(
          blockheight.into(),
          blockindex.try_into().unwrap(),
          outpoint.vout,
        )
        .unwrap(),
      );
    }

    let runestone = Runestone {
      freeze: Some(FreezeEdict {
        rune_id: Some(id),
        outpoints,
      }),
      ..default()
    };

    let script_pubkey = runestone.encipher();

    ensure!(
      script_pubkey.len() <= MAX_STANDARD_OP_RETURN_SIZE,
      "runestone greater than maximum OP_RETURN size: {} > {}",
      script_pubkey.len(),
      MAX_STANDARD_OP_RETURN_SIZE,
    );

    let unfunded_transaction = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: input,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
      }],
      output: vec![
        TxOut {
          script_pubkey,
          value: Amount::from_sat(0),
        },
        TxOut {
          script_pubkey: destination.script_pubkey(),
          value: postage,
        },
      ],
    };

    wallet.lock_non_cardinal_outputs()?;

    let unsigned_transaction =
      fund_raw_transaction(bitcoin_client, self.fee_rate, &unfunded_transaction)?;

    let signed_transaction = bitcoin_client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let signed_transaction = consensus::encode::deserialize(&signed_transaction)?;

    assert_eq!(
      Runestone::decipher(&signed_transaction),
      Some(Artifact::Runestone(runestone)),
    );

    let transaction = bitcoin_client.send_raw_transaction(&signed_transaction)?;

    Ok(Some(Box::new(Output {
      rune: self.rune,
      freezer: freezer_entry.spaced_rune,
      txid: transaction,
    })))
  }
}
