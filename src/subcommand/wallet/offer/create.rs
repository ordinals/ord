use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub psbt: String,
  pub seller_address: Address<NetworkUnchecked>,
  pub inscription: InscriptionId,
}

#[derive(Debug, Parser)]
pub(crate) struct Create {
  #[arg(long, help = "<INSCRIPTION> to make offer for.")]
  inscription: InscriptionId,
  #[arg(long, help = "<AMOUNT> to offer.")]
  amount: Amount,
  #[arg(long, help = "<FEE_RATE> for finalized transaction.")]
  fee_rate: FeeRate,
}

impl Create {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      !wallet.inscription_info().contains_key(&self.inscription),
      "inscription {} already in wallet",
      self.inscription
    );

    let Some(inscription) = wallet.get_inscription(self.inscription)? else {
      bail!("inscription {} does not exist", self.inscription);
    };

    let Some(postage) = inscription.value else {
      bail!("inscription {} unbound", self.inscription);
    };

    let Some(seller_address) = inscription.address else {
      bail!(
        "inscription {} script pubkey not valid address",
        self.inscription,
      );
    };

    let seller_address = seller_address
      .parse::<Address<NetworkUnchecked>>()
      .unwrap()
      .require_network(wallet.chain().network())?;

    let postage = Amount::from_sat(postage);

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
      inscription: self.inscription,
      seller_address: seller_address.into_unchecked(),
    })))
  }
}
