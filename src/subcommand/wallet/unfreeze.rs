use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Unfreeze {
  #[arg(long, help = "Don't sign or broadcast transaction")]
  dry_run: bool,
  #[clap(long, help = "Use <FEE_RATE> sats/vbyte for unfreeze transaction.")]
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
  pub outpoints: Vec<OutPoint>,
  pub txid: Txid,
  pub psbt: String,
  pub fee: u64,
}

impl Unfreeze {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let unsigned_transaction = wallet.create_unsigned_freeze_or_unfreeze_runes_transaction(
      false,
      self.fee_rate,
      self.rune,
      self.outpoints.clone(),
      self.postage,
    )?;

    let (txid, psbt, fee) =
      wallet.sign_and_broadcast_transaction(unsigned_transaction, self.dry_run, None)?;

    Ok(Some(Box::new(Output {
      rune: self.rune,
      outpoints: self.outpoints.clone(),
      txid,
      psbt,
      fee,
    })))
  }
}
