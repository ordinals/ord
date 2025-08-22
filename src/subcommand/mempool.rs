use super::*;
use crate::{settings::Settings, subcommand::SubcommandResult};
#[derive(Debug, Parser)]
pub(crate) struct Mempool {}

impl Mempool {
  pub fn run(self, settings: Settings) -> SubcommandResult {
    println!(
      "Mempool indexing is not yet implemented self: {:?} Settings: {:#?}",
      self, settings
    );
    Ok(None)
  }
}
