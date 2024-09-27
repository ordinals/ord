use super::*;

#[derive(Debug, Parser)]
pub(super) struct SharedArgs {
  #[arg(
    long,
    help = "Use <COMMIT_FEE_RATE> sats/vbyte for commit transaction.\nDefaults to <FEE_RATE> if unset."
  )]
  pub(crate) commit_fee_rate: Option<FeeRate>,
  #[arg(long, help = "Compress inscription content with brotli.")]
  pub(crate) compress: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB.")]
  pub(crate) fee_rate: FeeRate,
  #[arg(long, help = "Don't sign or broadcast transactions.")]
  pub(crate) dry_run: bool,
  #[arg(long, alias = "nobackup", help = "Do not back up recovery key.")]
  pub(crate) no_backup: bool,
  #[arg(
    long,
    alias = "nolimit",
    help = "Do not check that transactions are equal to or below the MAX_STANDARD_TX_WEIGHT of 400,000 weight units. Transactions over this limit are currently nonstandard and will not be relayed by bitcoind in its default configuration. Do not use this flag unless you understand the implications."
  )]
  pub(crate) no_limit: bool,
}
