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
    let mut env: BTreeMap<String, String> = BTreeMap::new();

    for (var, value) in env::vars_os() {
      let Some(var) = var.to_str() else {
        continue;
      };

      let Some(key) = var.strip_prefix("ORD_") else {
        continue;
      };

      env.insert(
        key.into(),
        value.into_string().map_err(|value| {
          anyhow!(
            "environment variable `{var}` not valid unicode: `{}`",
            value.to_string_lossy()
          )
        })?,
      );
    }

    let config = self.options.config()?;

    self
      .subcommand
      .run(Settings::new(self.options, env, config)?)
  }
}
