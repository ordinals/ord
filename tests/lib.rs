#![allow(clippy::type_complexity)]

use {
  self::{command_builder::CommandBuilder, expected::Expected, test_server::TestServer},
  bip39::Mnemonic,
  bitcoin::{blockdata::constants::COIN_VALUE, Network, OutPoint, Txid},
  executable_path::executable_path,
  pretty_assertions::assert_eq as pretty_assert_eq,
  regex::Regex,
  reqwest::{StatusCode, Url},
  serde::{de::DeserializeOwned, Deserialize},
  std::{
    fs,
    net::TcpListener,
    path::Path,
    process::{Child, Command, Stdio},
    str::{self, FromStr},
    thread,
    time::Duration,
  },
  tempfile::TempDir,
  test_bitcoincore_rpc::Sent,
};

macro_rules! assert_regex_match {
  ($string:expr, $pattern:expr $(,)?) => {
    let regex = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
    let string = $string;

    if !regex.is_match(string.as_ref()) {
      panic!(
        "Regex:\n\n{}\n\n…did not match string:\n\n{}",
        regex, string
      );
    }
  };
}

#[derive(Deserialize, Debug)]
struct Inscribe {
  #[allow(dead_code)]
  commit: Txid,
  inscription: String,
  reveal: Txid,
  fees: u64,
}

fn inscribe(rpc_server: &test_bitcoincore_rpc::Handle) -> Inscribe {
  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 foo.txt")
    .write("foo.txt", "FOO")
    .rpc_server(rpc_server)
    .output();

  rpc_server.mine_blocks(1);

  output
}

#[derive(Deserialize)]
struct Create {
  mnemonic: Mnemonic,
}

fn create_wallet(rpc_server: &test_bitcoincore_rpc::Handle) {
  CommandBuilder::new(format!("--chain {} wallet create", rpc_server.network()))
    .rpc_server(rpc_server)
    .output::<Create>();
}

mod command_builder;
mod core;
mod epochs;
mod expected;
mod find;
mod index;
mod info;
mod list;
mod parse;
mod server;
mod subsidy;
mod supply;
mod test_server;
mod traits;
mod version;
mod wallet;
