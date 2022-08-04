use {super::*, Network::*};

pub(crate) trait NetworkConfig {
  fn rpc_url(self) -> String;
  fn auth(self) -> Auth;
}

impl NetworkConfig for Network {
  fn rpc_url(self) -> String {
    format!(
      "127.0.0.1:{}",
      match self {
        Bitcoin => "8333",
        Regtest => "18443",
        Signet => "38333",
        Testnet => "18332",
      }
    )
  }

  fn auth(self) -> Auth {
    Auth::CookieFile(
      [
        if cfg!(macos) {
          String::from("~/Library/Application\\ Support/")
        } else if cfg!(windows) {
          format!("{}\\Bitcoin", env::var("APPDATA").unwrap())
        } else {
          String::from("~/.bitcoin")
        },
        self.to_string(),
        String::from(".cookie"),
      ]
      .iter()
      .collect(),
    )
  }
}
