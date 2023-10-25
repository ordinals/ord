use super::*;

pub mod balance;
pub mod issue;

#[derive(Debug, Parser)]
pub(crate) enum RunesSubcommand {
  #[command(about = "Issue a rune")]
  Issue(issue::Issue),
  #[command(about = "Issue a rune")]
  Balance(balance::Balance),
}

impl RunesSubcommand {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    match self {
      Self::Issue(issue) => issue.run(options),
      Self::Balance(balance) => balance.run(options),
    }
  }
}
