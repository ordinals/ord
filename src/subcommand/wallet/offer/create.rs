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
  #[arg(long, help = "Submit offer to <URL>.", value_name = "URL")]
  submit: Option<String>,
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

    // Bitcoin Core v24+ rejects a pre-selected input the wallet cannot solve
    // unless its weight is supplied via `input_weights`. The seller's input is
    // at the seller's address, which this wallet does not hold a key for, so
    // without this `fundrawtransaction` fails with "Not solvable pre-selected
    // input". Estimate the seller input weight the same way `wallet sweep` does
    // for its foreign inputs.
    let input_weights = {
      let mut witness = Witness::new();

      // public key
      witness.push([0; 33]);

      // signature
      witness.push([0; 73]);

      let weight = TxIn {
        previous_output: OutPoint::null(),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness,
      }
      .segwit_weight()
      .to_wu()
      .try_into()
      .unwrap();

      vec![fund_raw_transaction::InputWeight {
        txid: inscription.satpoint.outpoint.txid,
        vout: inscription.satpoint.outpoint.vout,
        weight,
      }]
    };

    let tx = fund_raw_transaction(
      wallet.bitcoin_client(),
      self.fee_rate,
      &tx,
      Some(input_weights),
    )?;

    let tx = consensus::encode::deserialize::<Transaction>(&tx)?;

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

    if let Some(url) = &self.submit {
      let response = reqwest::blocking::Client::new()
        .post(url)
        .body(result.psbt.as_bytes().to_vec())
        .send()
        .context("failed to submit PSBT")?;

      let status = response.status();

      let text = response
        .text()
        .context("failed to receive submission response body")?;

      ensure! {
        status == StatusCode::OK,
        "submission failed with status code {status}: {text}",
      }
    }

    Ok(Some(Box::new(Output {
      psbt: result.psbt,
      inscription: self.inscription,
      seller_address: seller_address.into_unchecked(),
    })))
  }
}
