#![allow(clippy::type_complexity)]

use {
  self::{
    command_builder::CommandBuilder, expected::Expected, slow_test::SlowTest, state::State,
    transaction_options::TransactionOptions,
  },
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
  bitcoin::{hash_types::Txid, network::constants::Network, Block, OutPoint, Script, Transaction},
  bitcoincore_rpc::{Client, RawTx, RpcApi},
  executable_path::executable_path,
  log::LevelFilter,
  regex::Regex,
  std::{
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

mod command_builder;
mod epochs;
mod expected;
mod find;
mod index;
mod info;
mod list;
mod parse;
mod range;
mod slow_test;
mod state;
mod supply;
mod traits;
mod transaction_options;
mod version;
