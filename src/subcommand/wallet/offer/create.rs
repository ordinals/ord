use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub psbt: String,
  pub seller_address: Address<NetworkUnchecked>,
  pub inscription: Option<InscriptionId>,
  pub rune: Option<Outgoing>,
}

#[derive(Debug, Parser)]
#[command(group = ArgGroup::new("target")
  .required(true)
  .multiple(false)
  .args(["inscription", "rune"]))]
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
}

impl Create {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let (seller_address, postage, utxo) = match (self.inscription, self.rune.clone()) {
      (Some(inscription_id), None) => {
        ensure!(
          !wallet.inscription_info().contains_key(&inscription_id),
          "inscription {} already in wallet",
          inscription_id
        );

        let Some(inscription) = wallet.get_inscription(inscription_id)? else {
          bail!("inscription {} does not exist", inscription_id);
        };

        if let Some(utxo) = self.utxo {
          ensure! {
            inscription.satpoint.outpoint == utxo,
            "inscription utxo {} does not match provided utxo {}",
            inscription.satpoint.outpoint,
            utxo
          };
        }

        let Some(postage) = inscription.value else {
          bail!("inscription {} unbound", inscription_id);
        };

        let Some(seller_address) = inscription.address else {
          bail!(
            "inscription {} script pubkey not valid address",
            inscription_id,
          );
        };

        let seller_address = seller_address
          .parse::<Address<NetworkUnchecked>>()
          .unwrap()
          .require_network(wallet.chain().network())?;

        (
          seller_address,
          Amount::from_sat(postage),
          inscription.satpoint.outpoint,
        )
      }
      (None, Some(outgoing)) => {
        let (decimal, spaced_rune) = match outgoing {
          Outgoing::Rune { decimal, rune } => (decimal, rune),
          _ => bail!("invalid format for --rune (must be `DECIMAL:RUNE`)"),
        };

        ensure!(
          wallet.has_rune_index(),
          "creating runes offer with `offer` requires index created with `--index-runes` flag",
        );

        wallet
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
          bail!("utxo {} does not hold any {} runes", utxo, spaced_rune);
        };

        let Some(pile) = runes.get(&spaced_rune) else {
          bail!("utxo {} does not hold any {} runes", utxo, spaced_rune);
        };

        ensure! {
          pile.amount == decimal.value,
          "utxo holds unexpected {} balance (expected {}, found {})",
          spaced_rune,
          decimal.value,
          pile.amount
        }

        ensure! {
          runes.len() == 1,
          "utxo {} holds multiple runes",
          utxo
        }

        let seller_address = seller_address.require_network(wallet.chain().network())?;

        (seller_address, Amount::from_sat(output_info.value), utxo)
      }
      _ => unreachable!("--inscription or --rune must be set, but not both"),
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
      output: vec![
        TxOut {
          value: postage,
          script_pubkey: wallet.get_change_address()?.into(),
        },
        TxOut {
          value: self.amount + postage,
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

    Ok(Some(Box::new(Output {
      psbt: result.psbt,
      seller_address: seller_address.into_unchecked(),
      inscription: self.inscription,
      rune: self.rune.clone(),
    })))
  }
}
