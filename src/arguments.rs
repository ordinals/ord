use {
  super::*,
  clap::builder::styling::{AnsiColor, Effects, Styles},
};

#[derive(Debug, Parser)]
#[command(
  version,
  styles = Styles::styled()
    .header(AnsiColor::Green.on_default() | Effects::BOLD)
    .usage(AnsiColor::Green.on_default() | Effects::BOLD)
    .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
    .placeholder(AnsiColor::Cyan.on_default()))
]
pub(crate) struct Arguments {
  #[command(flatten)]
  pub(crate) options: Options,
  #[command(subcommand)]
  pub(crate) subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> SubcommandResult {
    self.subcommand.run(self.options)
  }
}
