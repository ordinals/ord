use {
  arguments::Arguments,
  bitcoin::blockdata::transaction::OutPoint,
  bitcoincore_rpc::{Auth, Client, RpcApi},
  serde::Deserialize,
  std::collections::BTreeMap,
  structopt::StructOpt,
};

mod arguments;
mod catalog;
mod client;
mod price;
mod supply;

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn main() -> Result<()> {
  Arguments::from_args().run()
}
