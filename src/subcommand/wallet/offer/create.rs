use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub psbt: String,
  pub seller_address: Address<NetworkUnchecked>,
  pub inscription: Option<InscriptionId>,
  pub rune: Option<Outgoing>,
}

#[derive(Debug, Parser)]
pub(crate) struct Create {
  #[arg(long, help = "<INSCRIPTION> to make offer for.")]
  inscription: Option<InscriptionId>,
  #[arg(long, help = "<DECIMAL:RUNE> to make offer for.")]
  rune: Option<Outgoing>,
  #[arg(long, help = "<AMOUNT> to offer.")]
  amount: Amount,
  #[arg(long, help = "<FEE_RATE> for finalized transaction.")]
  fee_rate: FeeRate,
  #[arg(long, help = "UTXO to make an offer for. (format: <TXID:VOUT>)")]
  utxo: Option<OutPoint>,
  #[arg(
    long,
    help = "Include at least <AMOUNT> postage with receive output. [default: 10000sat]"
  )]
  postage: Option<Amount>,
}

impl Create {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let (psbt, seller_address) = match (self.inscription, self.rune.clone()) {
      (Some(inscription), None) => self.create_inscription_buy_offer(wallet, inscription)?,
      (None, Some(outgoing)) => self.create_rune_buy_offer(wallet, outgoing)?,
      (None, None) => bail!("must include either --inscription or --rune"),
      (Some(_), Some(_)) => bail!("cannot include both --inscription and --rune"),
    };

    Ok(Some(Box::new(Output {
      psbt,
      seller_address,
      inscription: self.inscription,
      rune: self.rune.clone(),
    })))
  }

  fn create_inscription_buy_offer(
    &self,
    wallet: Wallet,
    inscription_id: InscriptionId,
  ) -> Result<(String, Address<NetworkUnchecked>)> {
    ensure!(
      !wallet.inscription_info().contains_key(&inscription_id),
      "inscription {} already in wallet",
      inscription_id
    );

    let Some(inscription) = wallet.get_inscription(inscription_id)? else {
      bail!("inscription {} does not exist", inscription_id);
    };

    let Some(postage) = inscription.value else {
      bail!("inscription {} unbound", inscription_id);
    };

    let Some(seller_address) = inscription.address else {
      bail!(
        "inscription {} script pubkey not valid address",
        inscription_id,
      );
    };

    if let Some(utxo) = self.utxo {
      ensure! {
        inscription.satpoint.outpoint == utxo,
        "inscription utxo {} does not match provided utxo {}",
        inscription.satpoint.outpoint,
        utxo
      };
    }

    let seller_address = seller_address
      .parse::<Address<NetworkUnchecked>>()
      .unwrap()
      .require_network(wallet.chain().network())?;

    let seller_postage = Amount::from_sat(postage);

    let mut buyer_postage = self.postage.unwrap_or(TARGET_POSTAGE);

    if seller_postage > buyer_postage {
      buyer_postage = seller_postage;
    }

    let tx = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: inscription.satpoint.outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      }],
      output: vec![
        TxOut {
          value: buyer_postage,
          script_pubkey: wallet.get_change_address()?.into(),
        },
        TxOut {
          value: self.amount + seller_postage,
          script_pubkey: seller_address.clone().into(),
        },
      ],
    };

    let psbt = self.create_funded_buy_offer(wallet, tx)?;

    Ok((psbt, seller_address.into_unchecked()))
  }

  fn create_rune_buy_offer(
    &self,
    wallet: Wallet,
    outgoing: Outgoing,
  ) -> Result<(String, Address<NetworkUnchecked>)> {
    let (decimal, spaced_rune) = match outgoing {
      Outgoing::Rune { decimal, rune } => (decimal, rune),
      _ => bail!("invalid format for --rune (must be `DECIMAL:RUNE`)"),
    };

    ensure!(
      wallet.has_rune_index(),
      "creating runes offer with `buy-offer` requires index created with `--index-runes` flag",
    );

    let (id, _, _) = wallet
      .get_rune(spaced_rune.rune)?
      .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;

    let Some(utxo) = self.utxo else {
      bail!("--utxo must be set");
    };

    ensure!(
      !wallet.utxos().contains_key(&utxo),
      "utxo {} already in wallet",
      utxo
    );

    ensure! {
      wallet.output_exists(utxo)?,
      "utxo {} does not exist",
      utxo
    }

    let Some(output_info) = wallet.get_output_info(utxo)? else {
      bail!("utxo {} does not exist", utxo);
    };

    let Some(seller_address) = output_info.address else {
      bail!("utxo {} script pubkey not valid address", utxo);
    };

    let Some(runes) = output_info.runes else {
      bail!("utxo {} does not hold any runes", utxo);
    };

    let Some(pile) = runes.get(&spaced_rune) else {
      bail!("utxo {} does not hold any {} runes", utxo, spaced_rune);
    };

    if pile.amount < decimal.value {
      bail!(
        "utxo {} holds less {} than required ({} < {})",
        utxo,
        spaced_rune,
        pile.amount,
        decimal.value,
      );
    }

    if pile.amount > decimal.value {
      bail!(
        "utxo {} holds more {} than expected ({} > {})",
        utxo,
        spaced_rune,
        pile.amount,
        decimal.value,
      );
    }

    let buyer_postage = self.postage.unwrap_or(TARGET_POSTAGE);

    let seller_postage = Amount::from_sat(output_info.value);

    let seller_address = seller_address.require_network(wallet.chain().network())?;

    let output = if runes.len() > 1 {
      let runestone = Runestone {
        edicts: vec![Edict {
          amount: 0,
          id,
          output: 2,
        }],
        ..default()
      };

      vec![
        TxOut {
          value: seller_postage,
          script_pubkey: seller_address.clone().into(),
        },
        TxOut {
          value: self.amount,
          script_pubkey: seller_address.clone().into(),
        },
        TxOut {
          value: buyer_postage,
          script_pubkey: wallet.get_change_address()?.into(),
        },
        TxOut {
          value: Amount::ZERO,
          script_pubkey: runestone.encipher(),
        },
      ]
    } else {
      vec![
        TxOut {
          value: buyer_postage,
          script_pubkey: wallet.get_change_address()?.into(),
        },
        TxOut {
          value: self.amount + seller_postage,
          script_pubkey: seller_address.clone().into(),
        },
      ]
    };

    let tx = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: utxo,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      }],
      output,
    };

    let psbt = self.create_funded_buy_offer(wallet, tx)?;

    Ok((psbt, seller_address.into_unchecked()))
  }

  fn create_funded_buy_offer(&self, wallet: Wallet, tx: Transaction) -> Result<String> {
    wallet.lock_non_cardinal_outputs()?;

    let tx = fund_raw_transaction(wallet.bitcoin_client(), self.fee_rate, &tx)?;

    let tx = Transaction::consensus_decode(&mut tx.as_slice())?;

    let psbt = Psbt::from_unsigned_tx(tx)?;

    let result = wallet
      .bitcoin_client()
      .call::<String>("utxoupdatepsbt", &[base64_encode(&psbt.serialize()).into()])?;

    let result = wallet
      .bitcoin_client()
      .wallet_process_psbt(&result, Some(true), None, None)?;

    ensure! {
      !result.complete,
      "PSBT unexpectedly complete after processing with wallet",
    }

    Ok(result.psbt)
  }
}
