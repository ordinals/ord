#![allow(clippy::type_complexity)]

use {
  self::{command_builder::CommandBuilder, expected::Expected, test_server::TestServer},
  bitcoin::{
    address::{Address, NetworkUnchecked},
    blockdata::constants::COIN_VALUE,
    Network, OutPoint, Txid,
  },
  bitcoincore_rpc::bitcoincore_rpc_json::ListDescriptorsResult,
  chrono::{DateTime, Utc},
  executable_path::executable_path,
  ord::{
    api, chain::Chain, outgoing::Outgoing, subcommand::runes::RuneInfo, Edict, InscriptionId, Rune,
    RuneEntry, RuneId, Runestone,
  },
  ordinals::{Rarity, Sat, SatPoint},
  pretty_assertions::assert_eq as pretty_assert_eq,
  regex::Regex,
  reqwest::{StatusCode, Url},
  serde::de::DeserializeOwned,
  std::sync::Arc,
  std::{
    collections::BTreeMap,
    fs,
    io::Write,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::{self, FromStr},
    thread,
    time::Duration,
  },
  tempfile::TempDir,
  test_bitcoincore_rpc::TransactionTemplate,
};

macro_rules! assert_regex_match {
  ($value:expr, $pattern:expr $(,)?) => {
    let regex = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
    let string = $value.to_string();

    if !regex.is_match(string.as_ref()) {
      eprintln!("Regex did not match:");
      pretty_assert_eq!(regex.as_str(), string);
    }
  };
}

mod command_builder;
mod expected;
mod test_server;

mod balances;
mod decode;
mod epochs;
mod etch;
mod find;
mod index;
mod info;
mod json_api;
mod list;
mod parse;
mod runes;
mod server;
mod settings;
mod subsidy;
mod supply;
mod traits;
mod version;
mod wallet;

const RUNE: u128 = 99246114928149462;

type Inscribe = ord::wallet::inscribe::Output;
type Inscriptions = Vec<ord::subcommand::wallet::inscriptions::Output>;
type Etch = ord::subcommand::wallet::etch::Output;

fn create_wallet(bitcoin_rpc_server: &test_bitcoincore_rpc::Handle, ord_rpc_server: &TestServer) {
  CommandBuilder::new(format!(
    "--chain {} wallet create",
    bitcoin_rpc_server.network()
  ))
  .bitcoin_rpc_server(bitcoin_rpc_server)
  .ord_rpc_server(ord_rpc_server)
  .run_and_deserialize_output::<ord::subcommand::wallet::create::Output>();
}

fn inscribe(
  bitcoin_rpc_server: &test_bitcoincore_rpc::Handle,
  ord_rpc_server: &TestServer,
) -> (InscriptionId, Txid) {
  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(format!(
    "--chain {} wallet inscribe --fee-rate 1 --file foo.txt",
    bitcoin_rpc_server.network()
  ))
  .write("foo.txt", "FOO")
  .bitcoin_rpc_server(bitcoin_rpc_server)
  .ord_rpc_server(ord_rpc_server)
  .run_and_deserialize_output::<Inscribe>();

  bitcoin_rpc_server.mine_blocks(1);

  assert_eq!(output.inscriptions.len(), 1);

  (output.inscriptions[0].id, output.reveal)
}

fn etch(
  bitcoin_rpc_server: &test_bitcoincore_rpc::Handle,
  ord_rpc_server: &TestServer,
  rune: Rune,
) -> Etch {
  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(
    format!(
    "--index-runes --regtest wallet etch --rune {} --divisibility 0 --fee-rate 0 --supply 1000 --symbol ¢",
    rune
    )
  )
  .bitcoin_rpc_server(bitcoin_rpc_server)
  .ord_rpc_server(ord_rpc_server)
  .run_and_deserialize_output();

  bitcoin_rpc_server.mine_blocks(1);

  output
}

fn envelope(payload: &[&[u8]]) -> bitcoin::Witness {
  let mut builder = bitcoin::script::Builder::new()
    .push_opcode(bitcoin::opcodes::OP_FALSE)
    .push_opcode(bitcoin::opcodes::all::OP_IF);

  for data in payload {
    let mut buf = bitcoin::script::PushBytesBuf::new();
    buf.extend_from_slice(data).unwrap();
    builder = builder.push_slice(buf);
  }

  let script = builder
    .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
    .into_script();

  bitcoin::Witness::from_slice(&[script.into_bytes(), Vec::new()])
}

fn runes(rpc_server: &test_bitcoincore_rpc::Handle) -> BTreeMap<Rune, RuneInfo> {
  CommandBuilder::new("--index-runes --regtest runes")
    .bitcoin_rpc_server(rpc_server)
    .run_and_deserialize_output::<ord::subcommand::runes::Output>()
    .runes
}
