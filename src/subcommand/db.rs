// TODO: Deprecate

use redb::{Database, ReadableTable, Table, TableDefinition, WriteStrategy, WriteTransaction};

use super::*;

macro_rules! define_table {
  ($name:ident, $key:ty, $value:ty) => {
    const $name: TableDefinition<$key, $value> = TableDefinition::new(stringify!($name));
  };
}

define_table! { HEIGHT_TO_BLOCK_HASH, u64, &BlockHashValue }
define_table! { INSCRIPTION_ID_TO_INSCRIPTION_ENTRY, &InscriptionIdValue, InscriptionEntryValue }
define_table! { INSCRIPTION_ID_TO_SATPOINT, &InscriptionIdValue, &SatPointValue }
define_table! { INSCRIPTION_NUMBER_TO_INSCRIPTION_ID, u64, &InscriptionIdValue }
define_table! { OUTPOINT_TO_SAT_RANGES, &OutPointValue, &[u8] }
define_table! { OUTPOINT_TO_VALUE, &OutPointValue, u64}
define_table! { SATPOINT_TO_INSCRIPTION_ID, &SatPointValue, &InscriptionIdValue }
define_table! { SAT_TO_INSCRIPTION_ID, u64, &InscriptionIdValue }
define_table! { SAT_TO_SATPOINT, u64, &SatPointValue }
define_table! { STATISTIC_TO_COUNT, u64, u64 }
define_table! { WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP, u64, u128 }

#[derive(Debug, Parser)]
pub(crate) struct Db {
//   #[clap(long)]
//   transactions: bool,
}


impl Db {
  pub(crate) fn run(self) -> Result {
    println!("Debug DB Store");

    let db_path = PathBuf::from_str("/root/.local/share/ord/testnet3/index.redb");

    match unsafe { Database::builder().open_mmapped(&db_path.unwrap()) } {
      Ok(database) => {
        println!("DB open");

        let tables = database.begin_read()?.list_tables().unwrap();
        tables.for_each(|table| {
          println!("==========================");
          println!("table: {}", table);
          println!("==========================");
        });

        let m_tables = database.begin_read()?.list_multimap_tables().unwrap();
        m_tables.for_each(|m_table| {
          println!("==========================");
          println!("m_table:{}", m_table);
          println!("==========================");
        });

        let length = database
            .begin_read()?
            .open_table(HEIGHT_TO_BLOCK_HASH)?
            .len();
        println!("-------------------------");
        println!("{}", length.unwrap());
        println!("-------------------------");

        let _ = database
            .begin_read()?
            .open_table(SAT_TO_INSCRIPTION_ID)?
            .get(0)?
            .map(|r| {
                println!("===========================");
                println!("{:?}", r.value());
                println!("===========================");
            });
        

        // let index = Index::open(None)?;
        // let inscriptions = index.get_inscriptions(None)?;

        // println!("||||||||||||||||||||||||");
        // println!("{:?}", inscriptions);
        // println!("||||||||||||||||||||||||");

        // println!("||||||||||||||||||||||");
        // println!("{:?}", sat_to_inscription.unwrap().unwrap().value());
        // println!("||||||||||||||||||||||");

        // let index: [u8; 36] = [2432793; 36];

        // database
        //     .begin_read()?
        //     .open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY)?
        //     .get(&index)
        //     .map(|r| {
        //         println!("===========================");
        //         println!("{:?}", r.unwrap().value().0);
        //         println!("===========================");
        //     });
      }
      Err(_) => {
        println!("DB error");    
      },
    };

    Ok(())
  }
}

pub(super) trait Entry: Sized {
  type Value;

  fn load(value: Self::Value) -> Self;

  fn store(self) -> Self::Value;
}

pub(super) type BlockHashValue = [u8; 32];

impl Entry for BlockHash {
  type Value = BlockHashValue;

  fn load(value: Self::Value) -> Self {
    BlockHash::from_inner(value)
  }

  fn store(self) -> Self::Value {
    self.into_inner()
  }
}

pub(crate) struct InscriptionEntry {
  pub(crate) fee: u64,
  pub(crate) height: u64,
  pub(crate) number: u64,
  pub(crate) sat: Option<Sat>,
  pub(crate) timestamp: u32,
}

pub(crate) type InscriptionEntryValue = (u64, u64, u64, u64, u32);

impl Entry for InscriptionEntry {
  type Value = InscriptionEntryValue;

  fn load((fee, height, number, sat, timestamp): InscriptionEntryValue) -> Self {
    Self {
      fee,
      height,
      number,
      sat: if sat == u64::MAX {
        None
      } else {
        Some(Sat(sat))
      },
      timestamp,
    }
  }

  fn store(self) -> Self::Value {
    (
      self.fee,
      self.height,
      self.number,
      match self.sat {
        Some(sat) => sat.n(),
        None => u64::MAX,
      },
      self.timestamp,
    )
  }
}

pub(super) type InscriptionIdValue = [u8; 36];

impl Entry for InscriptionId {
  type Value = InscriptionIdValue;

  fn load(value: Self::Value) -> Self {
    let (txid, index) = value.split_at(32);
    Self {
      txid: Txid::from_inner(txid.try_into().unwrap()),
      index: u32::from_be_bytes(index.try_into().unwrap()),
    }
  }

  fn store(self) -> Self::Value {
    let mut value = [0; 36];
    let (txid, index) = value.split_at_mut(32);
    txid.copy_from_slice(self.txid.as_inner());
    index.copy_from_slice(&self.index.to_be_bytes());
    value
  }
}

pub(super) type OutPointValue = [u8; 36];

impl Entry for OutPoint {
  type Value = OutPointValue;

  fn load(value: Self::Value) -> Self {
    Decodable::consensus_decode(&mut io::Cursor::new(value)).unwrap()
  }

  fn store(self) -> Self::Value {
    let mut value = [0; 36];
    self.consensus_encode(&mut value.as_mut_slice()).unwrap();
    value
  }
}

pub(super) type SatPointValue = [u8; 44];

impl Entry for SatPoint {
  type Value = SatPointValue;

  fn load(value: Self::Value) -> Self {
    Decodable::consensus_decode(&mut io::Cursor::new(value)).unwrap()
  }

  fn store(self) -> Self::Value {
    let mut value = [0; 44];
    self.consensus_encode(&mut value.as_mut_slice()).unwrap();
    value
  }
}

pub(super) type SatRange = (u64, u64);

impl Entry for SatRange {
  type Value = [u8; 11];

  fn load([b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10]: Self::Value) -> Self {
    let raw_base = u64::from_le_bytes([b0, b1, b2, b3, b4, b5, b6, 0]);

    // 51 bit base
    let base = raw_base & ((1 << 51) - 1);

    let raw_delta = u64::from_le_bytes([b6, b7, b8, b9, b10, 0, 0, 0]);

    // 33 bit delta
    let delta = raw_delta >> 3;

    (base, base + delta)
  }

  fn store(self) -> Self::Value {
    let base = self.0;
    let delta = self.1 - self.0;
    let n = u128::from(base) | u128::from(delta) << 51;
    n.to_le_bytes()[0..11].try_into().unwrap()
  }
}
