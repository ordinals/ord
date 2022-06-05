#![allow(clippy::too_many_arguments)]

use {
  crate::{
    arguments::Arguments, bytes::Bytes, decode_bech32::decode_bech32, epoch::Epoch, height::Height,
    index::Index, nft::Nft, options::Options, ordinal::Ordinal, sat_point::SatPoint,
    subcommand::Subcommand,
  },
  anyhow::{anyhow, Context, Error},
  axum::{extract, http::StatusCode, response::IntoResponse, routing::get, Json, Router},
  axum_server::Handle,
  bech32::{FromBase32, ToBase32},
  bitcoin::{
    blockdata::constants::COIN_VALUE, consensus::Decodable, consensus::Encodable, Address, Block,
    BlockHash, Network, OutPoint, Transaction, Txid,
  },
  bitcoin_hashes::{sha256d, Hash, HashEngine},
  chrono::{DateTime, NaiveDateTime, Utc},
  clap::Parser,
  derive_more::{Display, FromStr},
  integer_cbrt::IntegerCubeRoot,
  integer_sqrt::IntegerSquareRoot,
  lazy_static::lazy_static,
  qrcode_generator::QrCodeEcc,
  secp256k1::{rand, schnorr::Signature, KeyPair, Secp256k1, SecretKey, XOnlyPublicKey},
  serde::{Deserialize, Serialize},
  std::{
    cmp::Ordering,
    collections::VecDeque,
    env,
    fmt::{self, Display, Formatter},
    fs,
    io::{self, BufRead, Write},
    net::ToSocketAddrs,
    ops::{Add, AddAssign, Deref, Sub},
    path::PathBuf,
    process,
    str::FromStr,
    sync::{
      atomic::{self, AtomicU64},
      Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
  },
  tokio::runtime::Runtime,
  tower_http::cors::{Any, CorsLayer},
};

#[cfg(feature = "redb")]
use redb_database::{Database, WriteTransaction};

#[cfg(not(feature = "redb"))]
use lmdb_database::{Database, WriteTransaction};

mod arguments;
mod bytes;
mod decode_bech32;
mod epoch;
mod height;
mod index;
#[cfg(not(feature = "redb"))]
mod lmdb_database;
mod nft;
mod options;
mod ordinal;
#[cfg(feature = "redb")]
mod redb_database;
mod sat_point;
mod subcommand;

type Result<T = (), E = Error> = std::result::Result<T, E>;

static INTERRUPTS: AtomicU64 = AtomicU64::new(0);

lazy_static! {
  static ref LISTENERS: Mutex<Vec<Handle>> = Mutex::new(Vec::new());
}

fn main() {
  env_logger::init();

  ctrlc::set_handler(move || {
    LISTENERS
      .lock()
      .unwrap()
      .iter()
      .for_each(|handle| handle.graceful_shutdown(Some(Duration::from_millis(100))));

    let interrupts = INTERRUPTS.fetch_add(1, atomic::Ordering::Relaxed);

    if interrupts > 5 {
      process::exit(1);
    }
  })
  .expect("Error setting ctrl-c handler");

  if let Err(err) = Arguments::parse().run() {
    eprintln!("error: {}", err);
    err
      .chain()
      .skip(1)
      .for_each(|cause| eprintln!("because: {}", cause));
    if env::var_os("RUST_BACKTRACE")
      .map(|val| val == "1")
      .unwrap_or_default()
    {
      eprintln!("{}", err.backtrace());
    }
    process::exit(1);
  }
}
