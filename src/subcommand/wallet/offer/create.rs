use super::*;

// TODO:
// - [ ] add a --dry-run flag that doesn't sign our inputs in the PSBT
// - [ ] pub fee: u64,

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
      "{} in our wallet",
      self.inscription
    );

    let inscription = wallet.get_inscription(self.inscription)?;

    let utxo = inscription.satpoint.outpoint;

    let Some(seller_address) = inscription.address else {
      bail!("{} not owned by an address", self.inscription);
    };

    let Ok(seller_address) = Address::from_str(&seller_address) else {
      bail!("{} not owned by an usable address", self.inscription);
    };

    let Some(postage) = inscription.value else {
      bail!("inscription is unbound");
    };

    let unsigned_transaction = Transaction {
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
          value: Amount::from_sat(postage),
          script_pubkey: wallet.get_change_address()?.into(),
        },
        TxOut {
          value: self.amount,
          script_pubkey: seller_address
            .clone()
            .require_network(wallet.chain().network())?
            .into(),
        },
      ],
    };

    let unsigned_transaction_hex = fund_raw_transaction(
      wallet.bitcoin_client(),
      self.fee_rate,
      &unsigned_transaction,
    )?;

    let unsigned_transaction =
      Transaction::consensus_decode(&mut unsigned_transaction_hex.as_slice())?;

    let unsigned_psbt = Psbt::from_unsigned_tx(unsigned_transaction.clone())?;

    let encoded_psbt = base64::engine::general_purpose::STANDARD.encode(unsigned_psbt.serialize());

    let result = wallet
      .bitcoin_client()
      .call::<String>("utxoupdatepsbt", &[encoded_psbt.into()])?;

    let result = wallet
      .bitcoin_client()
      .wallet_process_psbt(&result, Some(true), None, None)?;

    let signed_tx = wallet
      .bitcoin_client()
      .sign_raw_transaction_with_wallet(&unsigned_transaction.clone(), None, None)?
      .transaction()?;

    let mut final_psbt =
      Psbt::deserialize(&base64::engine::general_purpose::STANDARD.decode(result.psbt)?)?;

    final_psbt.inputs[1].final_script_witness = Some(signed_tx.input[1].witness.clone());

    Ok(Some(Box::new(Output {
      psbt: base64::engine::general_purpose::STANDARD.encode(final_psbt.serialize()),
      inscription: self.inscription,
      seller_address,
    })))
  }
}
