use super::*;

// todo:
// - assert that only owned UTXOs in PSBT is our inscription

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub txid: Txid,
}

#[derive(Debug, Parser)]
pub(crate) struct Accept {
  #[arg(long, help = "Accept <PSBT>")]
  psbt: String,
}

impl Accept {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let psbt = wallet
      .bitcoin_client()
      .wallet_process_psbt(&self.psbt, Some(true), None, None)?
      .psbt;

    let signed_tx = wallet
      .bitcoin_client()
      .finalize_psbt(&psbt, None)?
      .hex
      .ok_or_else(|| anyhow!("unable to sign transaction"))?;

    let txid = wallet.send_raw_transaction(&signed_tx, None)?;

    Ok(Some(Box::new(Output { txid })))
  }
}
