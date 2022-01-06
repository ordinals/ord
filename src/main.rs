use {
  arguments::Arguments,
  bitcoin::{blockdata::constants::genesis_block, consensus::Decodable, Block, Network},
  redb::{
    Database, MultimapTable, ReadOnlyMultimapTable, ReadOnlyTable, ReadableMultimapTable,
    ReadableTable, Table,
  },
  std::{fs, ops::Deref},
  structopt::StructOpt,
};

mod arguments;

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn main() -> Result<()> {
  Arguments::from_args().run()
}
