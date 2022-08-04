use super::*;

#[derive(Parser)]
#[clap(version)]
pub(crate) struct Arguments {
  #[clap(flatten)]
  options: Options,
  #[clap(subcommand)]
  subcommand: Subcommand,
}

impl Arguments {
  pub(crate) fn run(self) -> Result<()> {
    self.subcommand.run(self.options)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rpc_url_overrides_network() {
    assert_eq!(
      Arguments::try_parse_from(&[
        "ord",
        "--rpc-url=127.0.0.1:1234",
        "--network=signet",
        "index"
      ])
      .unwrap()
      .options
      .rpc_url(),
      "127.0.0.1:1234"
    );
  }

  #[test]
  fn cookie_file_overrides_network() {
    assert_eq!(
      Arguments::try_parse_from(&["ord", "--cookie-file=/foo/bar", "--network=signet", "index"])
        .unwrap()
        .options
        .auth()
        .unwrap(),
      Auth::CookieFile(PathBuf::from("/foo/bar"))
    );
  }

  #[test]
  fn uses_network_defaults() {
    let arguments = Arguments::try_parse_from(&["ord", "--network=signet", "index"]).unwrap();

    assert_eq!(arguments.options.rpc_url(), "127.0.0.1:38333");

    assert!(arguments.options.auth().is_ok())
  }
}
