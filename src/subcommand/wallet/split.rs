use super::*;

// todo:
// - test that duplicate keys are an error
// - select runes first, may contain for sats
// - check for dust outputs
//
// - libre relay
//   - allow 4 meggers

struct Splitfile {
  splits: Vec<SplitOutput>,
}

struct SplitOutput {
  address: Option<Address<NetworkUnchecked>>,
  amount: Amount,
  runes: BTreeMap<SpacedRune, Decimal>,
}

#[derive(Debug, Parser)]
pub(crate) struct Split {
  #[arg(long, help = "Don't sign or broadcast transaction")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
  #[arg(
    long,
    help = "Target <AMOUNT> postage with sent inscriptions. [default: 10000 sat]",
    value_name = "AMOUNT"
  )]
  pub(crate) postage: Option<Amount>,
  pub(crate) split: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {}

impl Split {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    todo!()
  }
}
