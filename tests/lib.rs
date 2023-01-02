#![allow(clippy::type_complexity)]

use {
  self::{command_builder::CommandBuilder, expected::Expected, test_server::TestServer},
  bitcoin::{blockdata::constants::COIN_VALUE, Address, Network, OutPoint, Txid},
  executable_path::executable_path,
  pretty_assertions::assert_eq as pretty_assert_eq,
  regex::Regex,
  reqwest::{StatusCode, Url},
  std::{
    fs,
    net::TcpListener,
    path::Path,
    process::{Child, Command, Stdio},
    str, thread,
    time::Duration,
  },
  tempfile::TempDir,
  test_bitcoincore_rpc::Sent,
  unindent::Unindent,
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

fn reveal_txid_from_inscribe_stdout(stdout: &str) -> Txid {
  stdout
    .lines()
    .nth(1)
    .unwrap()
    .split('\t')
    .nth(1)
    .unwrap()
    .parse()
    .unwrap()
}

fn create_inscription(rpc_server: &test_bitcoincore_rpc::Handle, filename: &str) -> Txid {
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 {filename}"
  ))
  .write(filename, "HELLOWORLD")
  .rpc_server(rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  inscription_id
}

mod command_builder;
mod epochs;
mod expected;
mod find;
mod index;
mod info;
mod list;
mod parse;
mod preview;
mod server;
mod subsidy;
mod supply;
mod test_server;
mod traits;
mod version;
mod wallet;
