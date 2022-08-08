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
  use {super::*, std::path::Path};

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
        .cookie_file()
        .unwrap(),
      Path::new("/foo/bar")
    );
  }

  #[test]
  fn use_default_network() {
    let arguments = Arguments::try_parse_from(&["ord", "index"]).unwrap();

    assert_eq!(arguments.options.rpc_url(), "127.0.0.1:8333");

    assert!(arguments
      .options
      .cookie_file()
      .unwrap()
      .ends_with(".cookie"));
  }

  #[test]
  fn uses_network_defaults() {
    let arguments = Arguments::try_parse_from(&["ord", "--network=signet", "index"]).unwrap();

    assert_eq!(arguments.options.rpc_url(), "127.0.0.1:38333");

    assert!(arguments
      .options
      .cookie_file()
      .unwrap()
      .display()
      .to_string()
      .ends_with("/signet/.cookie"))
  }

  #[test]
  fn mainnet_cookie_file_path() {
    let arguments = Arguments::try_parse_from(&["ord", "index"]).unwrap();

    let cookie_file = arguments
      .options
      .cookie_file()
      .unwrap()
      .display()
      .to_string();

    if cfg!(target_os = "linux") {
      assert!(cookie_file.ends_with("/.bitcoin/.cookie"));
    } else {
      assert!(cookie_file.ends_with("/Bitcoin/.cookie"));
    }
  }

  #[test]
  fn othernet_cookie_file_path() {
    let arguments = Arguments::try_parse_from(&["ord", "--network=signet", "index"]).unwrap();

    let cookie_file = arguments
      .options
      .cookie_file()
      .unwrap()
      .display()
      .to_string();

    if cfg!(target_os = "linux") {
      assert!(cookie_file.ends_with("/.bitcoin/signet/.cookie"));
    } else {
      assert!(cookie_file.ends_with("/Bitcoin/signet/.cookie"));
    }
  }
}
