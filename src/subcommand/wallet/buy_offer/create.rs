use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub psbt: String,
  pub seller_address: Address<NetworkUnchecked>,
  pub outgoing: Outgoing,
  pub amount: Amount,
}

#[derive(Debug, Parser)]
pub(crate) struct Create {
  #[arg(long, help = "<INSCRIPTION> or <DECIMAL:RUNE> to make offer for.")]
  pub outgoing: Outgoing,
  #[arg(long, help = "<AMOUNT> to offer.")]
  pub amount: Amount,
  #[arg(long, help = "<FEE_RATE> for finalized transaction.")]
  pub fee_rate: FeeRate,
  #[arg(long, help = "UTXO to make an offer for. (format: <TXID:VOUT>)")]
  pub outpoint: Option<OutPoint>,
  #[arg(
    long,
    help = "Include <AMOUNT> postage with receive output. [default: 10000sat]"
  )]
  pub postage: Option<Amount>,
}

impl Create {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let (psbt, seller_address) = match self.outgoing {
      Outgoing::InscriptionId(inscription_id) => {
        self.create_inscription_buy_offer(wallet, inscription_id)?
      }
      Outgoing::Rune { decimal, rune } => self.create_rune_buy_offer(wallet, decimal, rune)?,
      _ => bail!("outgoing must be either <INSCRIPTION> or <DECIMAL:RUNE>"),
    };

    Ok(Some(Box::new(Output {
      psbt,
      seller_address,
      outgoing: self.outgoing.clone(),
      amount: self.amount,
    })))
  }

  pub fn create_inscription_buy_offer(
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

    if let Some(outpoint) = self.outpoint {
      ensure! {
        inscription.satpoint.outpoint == outpoint,
        "inscription outpoint {} does not match provided outpoint {}",
        inscription.satpoint.outpoint,
        outpoint
      };
    }

    let seller_address = seller_address
      .parse::<Address<NetworkUnchecked>>()
      .unwrap()
      .require_network(wallet.chain().network())?;

    let seller_postage = Amount::from_sat(postage);

    self.create_buy_offer(
      wallet,
      inscription.satpoint.outpoint,
      seller_postage,
      seller_address,
    )
  }

  fn create_rune_buy_offer(
    &self,
    wallet: Wallet,
    decimal: Decimal,
    spaced_rune: SpacedRune,
  ) -> Result<(String, Address<NetworkUnchecked>)> {
    let Some(outpoint) = self.outpoint else {
      bail!("--outpoint must be set");
    };

    ensure!(
      !wallet.output_info().contains_key(&outpoint),
      "outpoint {} already in wallet",
      outpoint
    );

    let Some(output_info) = wallet.get_any_output_info(outpoint)? else {
      bail!("outpoint {} does not exist", outpoint);
    };

    let Some(runes) = output_info.runes else {
      bail!("outpoint {} does not hold any runes", outpoint);
    };

    let Some(pile) = runes.get(&spaced_rune) else {
      bail!(
        "outpoint {} does not hold any {} runes",
        outpoint,
        spaced_rune
      );
    };

    ensure! {
      runes.len() == 1,
      "outpoint {} holds multiple runes",
      outpoint
    };

    if pile.amount < decimal.value {
      bail!(
        "outpoint {} holds less {} than required ({} < {})",
        outpoint,
        spaced_rune,
        pile.amount,
        decimal.value,
      );
    }

    if pile.amount > decimal.value {
      bail!(
        "outpoint {} holds more {} than expected ({} > {})",
        outpoint,
        spaced_rune,
        pile.amount,
        decimal.value,
      );
    }

    let Some(seller_address) = output_info.address else {
      bail!("outpoint {} script pubkey not valid address", outpoint,);
    };

    let seller_address = seller_address.require_network(wallet.chain().network())?;

    let seller_postage = Amount::from_sat(output_info.value);

    self.create_buy_offer(wallet, outpoint, seller_postage, seller_address)
  }

  fn create_buy_offer(
    &self,
    wallet: Wallet,
    outpoint: OutPoint,
    seller_postage: Amount,
    seller_address: Address,
  ) -> Result<(String, Address<NetworkUnchecked>)> {
    let buyer_postage = self.postage.unwrap_or(TARGET_POSTAGE);

    let tx = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: outpoint,
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

    Ok((result.psbt, seller_address.into_unchecked()))
  }
}
