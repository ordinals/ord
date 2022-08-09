#![allow(clippy::type_complexity)]

use {
  self::{state::State, test::Test},
  bdk::{
    blockchain::{
      rpc::{RpcBlockchain, RpcConfig},
      ConfigurableBlockchain,
    },
    database::MemoryDatabase,
    keys::bip39::Mnemonic,
    template::Bip84,
    wallet::{signer::SignOptions, AddressIndex, SyncOptions, Wallet},
    KeychainKind,
  },
  bitcoin::hash_types::Txid,
  bitcoin::{network::constants::Network, Block, OutPoint},
  bitcoincore_rpc::{Client, RawTx, RpcApi},
  executable_path::executable_path,
  log::LevelFilter,
  regex::Regex,
  std::{
    collections::BTreeMap,
    fs,
    net::TcpListener,
    process::{Child, Command, Stdio},
    str,
    sync::Once,
    thread::sleep,
    time::Duration,
  },
  tempfile::TempDir,
  unindent::Unindent,
};

mod epochs;
mod find;
mod index;
mod info;
mod list;
mod name;
mod nft;
mod range;
mod server;
mod state;
mod supply;
mod test;
mod traits;
mod version;
mod wallet;

fn free_port() -> u16 {
  TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port()
}

#[derive(Debug)]
enum Expected {
  String(String),
  Regex(Regex),
  Ignore,
}

impl Expected {
  fn regex(pattern: &str) -> Self {
    Self::Regex(Regex::new(&format!("^(?s){}$", pattern)).unwrap())
  }

  fn assert_match(&self, output: &str) {
    match self {
      Self::String(string) => assert_eq!(output, string),
      Self::Regex(regex) => assert!(
        regex.is_match(output),
        "output did not match regex: {:?}",
        output
      ),
      Self::Ignore => {}
    }
  }
}

struct Output {
  state: State,
  stdout: String,
}

struct TransactionOptions<'a> {
  slots: &'a [(usize, usize, usize)],
  output_count: usize,
  fee: u64,
}
