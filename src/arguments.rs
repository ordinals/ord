use {
  super::*,
  clap::builder::styling::{AnsiColor, Effects, Styles},
};

#[derive(Debug, Parser)]
#[command(
  version,
  styles = Styles::styled()
    .error(AnsiColor::Red.on_default() | Effects::BOLD)
    .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
    .invalid(AnsiColor::Red.on_default())
    .literal(AnsiColor::Blue.on_default())
    .placeholder(AnsiColor::Cyan.on_default())
    .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
    .valid(AnsiColor::Green.on_default()),
)]
pub(crate) struct Arguments {
  #[command(flatten)]
  pub(crate) options: Options,
  #[command(subcommand)]
  pub(crate) subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> SnafuResult<Option<Box<dyn subcommand::Output>>> {
    let mut env: BTreeMap<String, String> = BTreeMap::new();

    for (variable, value) in env::vars_os() {
      let Some(variable) = variable.to_str() else {
        continue;
      };

      let Some(key) = variable.strip_prefix("ORD_") else {
        continue;
      };

      env.insert(
        key.into(),
        value
          .into_string()
          .map_err(|value| SnafuError::EnvVarUnicode {
            backtrace: Backtrace::capture(),
            value,
            variable: variable.into(),
          })?,
      );
    }

    Ok(self.subcommand.run(Settings::load(self.options)?)?)
  }
}
