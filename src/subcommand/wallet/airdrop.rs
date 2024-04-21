use super::*;

// read in TSV of addresses
// verify all addresses
//

#[derive(Debug, Parser)]
pub(crate) struct Airdrop {
  #[clap(long, help = "Use <FEE_RATE> sats/vbyte for mint transaction.")]
  fee_rate: FeeRate,
  #[clap(long, help = "Airdrop <RUNE>. May contain `.` or `•`as spacers.")]
  rune: SpacedRune,
  #[clap(
    long,
    help = "Include <AMOUNT> postage with airdrop output. [default: 10000sat]"
  )]
  postage: Option<Amount>,
  #[clap(
    long,
    help = "Send minted runes to addresses listed in <DESTINATIONS> tsv file."
  )]
  destinations: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
  pub rune: SpacedRune,
  pub pile: Pile,
  pub txid: Txid,
}

impl Airdrop {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "`ord wallet mint` requires index created with `--index-runes` flag",
    );

    let rune = self.rune.rune;

    let bitcoin_client = wallet.bitcoin_client();

    let destinations = fs::read_to_string(self.destinations.clone())
      .with_context(|| format!("I/O error reading `{}`", self.destinations.display()))?;

    Ok(None)
  }
}
