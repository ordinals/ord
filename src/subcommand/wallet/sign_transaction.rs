use {super::*, base64::Engine, bitcoin::psbt::Psbt};

pub(super) fn sign_transaction(
  wallet: &Wallet,
  unsigned_transaction: Transaction,
  dry_run: bool,
) -> Result<(Txid, String, u64)> {
  let unspent_outputs = wallet.utxos();

  let (txid, psbt) = if dry_run {
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

  Ok((txid, psbt, fee))
}
