#![allow(clippy::type_complexity)]

use {
  self::{command_builder::CommandBuilder, expected::Expected, test_server::TestServer},
  bitcoin::{
    address::{Address, NetworkUnchecked},
    blockdata::constants::COIN_VALUE,
    Network, OutPoint, Txid,
  },
  executable_path::executable_path,
  ord::{
    inscription_id::InscriptionId,
    rarity::Rarity,
    templates::{
      block::BlockJson, inscription::InscriptionJson, inscriptions::InscriptionsJson,
      output::OutputJson, sat::SatJson,
    },
    SatPoint,
  },
  pretty_assertions::assert_eq as pretty_assert_eq,
  regex::Regex,
  reqwest::{StatusCode, Url},
  serde::de::DeserializeOwned,
  std::{
    collections::BTreeMap,
    fs,
    io::Write,
    net::TcpListener,
    path::Path,
    process::{Child, Command, Stdio},
    str::{self, FromStr},
    thread,
    time::Duration,
  },
  tempfile::TempDir,
  test_bitcoincore_rpc::{Sent, TransactionTemplate},
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

type Inscribe = ord::subcommand::wallet::inscribe::Output;

fn inscribe(rpc_server: &test_bitcoincore_rpc::Handle) -> (InscriptionId, Txid) {
  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscribe --fee-rate 1 --file foo.txt")
    .write("foo.txt", "FOO")
    .rpc_server(rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(output.inscriptions.len(), 1);

  (output.inscriptions[0].id, output.reveal)
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

fn create_wallet(rpc_server: &test_bitcoincore_rpc::Handle) {
  CommandBuilder::new(format!("--chain {} wallet create", rpc_server.network()))
    .rpc_server(rpc_server)
    .run_and_deserialize_output::<ord::subcommand::wallet::create::Output>();
}

mod command_builder;
mod expected;
mod test_server;

mod core;
mod decode;
mod epochs;
mod find;
mod index;
mod info;
mod json_api;
mod list;
mod parse;
mod server;
mod subsidy;
mod supply;
mod traits;
mod version;
mod wallet;
