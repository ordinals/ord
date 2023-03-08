use super::*;


#[derive(Debug, Parser)]
pub(crate) struct Offer {
  pub(crate) inscription_id: InscriptionId,
  #[clap(
    long,
    default_value = "1.0",
    help = "Use fee rate of <FEE_RATE> sats/vB"
  )]
  pub(crate) fee_rate: FeeRate,
  pub(crate) dry_run: bool,
}

impl Offer {
  pub(crate) fn run(self, options: Options) -> Result {
    Ok(())
  }
}
