use super::*;

// use lmdb::{
//   Cursor, Database as Table, DatabaseFlags,  Error, RwTransaction, Transaction,
//   WriteFlags,
// };

use lmdb_zero::{self as lmdb, EnvBuilder, Environment};

use std::{fs, path::Path};

const HEIGHT_TO_HASH: &'static str = "HEIGHT_TO_HASH";
const OUTPOINT_TO_ORDINAL_RANGES: &'static str = "OUTPOINT_TO_ORDINAL_RANGES";
const KEY_TO_SATPOINT: &'static str = "KEY_TO_SATPOINT";

pub(crate) struct Database(Environment);

impl Database {
  pub(crate) fn open(options: &Options) -> Result<Self> {
    let path = "index.lmdb";

    fs::create_dir_all(path)?;

    let env = unsafe {
      let mut builder = EnvBuilder::new()?;

      builder.set_maxdbs(3);
      builder.set_mapsize(options.index_size.0);

      builder
        .open(path, lmdb::open::Flags::empty(), 0o600)
        .unwrap()
    };

    Ok(Self(env))
  }

  pub(crate) fn begin_write(&self) -> Result<WriteTransaction> {
    Ok(WriteTransaction::new(&self.0)?)
  }

  pub(crate) fn print_info(&self) -> Result {
    // let tx = self.begin_write()?;

    // let blocks_indexed = tx.height()?;

    // let outputs_indexed = tx.outputs_indexed()?;

    // tx.abort()?;

    // let stats = self.0.stat()?;

    // println!("blocks indexed: {}", blocks_indexed);
    // println!("outputs indexed: {}", outputs_indexed);
    // println!("tree height: {}", stats.depth());
    // // println!("free pages: {}", stats.free_pages());
    // println!(
    //   "stored: {}",
    //   Bytes(
    //     (stats.branch_pages() + stats.leaf_pages() + stats.overflow_pages())
    //       * stats.page_size() as usize
    //   )
    // );
    // // println!("overhead: {}", Bytes(stats.overhead_bytes()));
    // // println!("fragmented: {}", Bytes(stats.fragmented_bytes()));
    // println!(
    //   "index size: {}",
    //   Bytes(std::fs::metadata("index.lmdb")?.len().try_into()?)
    // );

    // Ok(())
    todo!()
  }

  pub(crate) fn find(&self, ordinal: Ordinal) -> Result<Option<(u64, u64, SatPoint)>> {
    // let height_to_hash = self.0.open_db(Some(HEIGHT_TO_HASH))?;

    // let _key_to_satpoint = self.0.open_db(Some(KEY_TO_SATPOINT))?;

    // let rtx = self.0.begin_ro_txn()?;

    // match rtx.open_ro_cursor(height_to_hash)?.iter().last() {
    //   Some(result) if u64::from_be_bytes(result?.0.try_into()?) >= ordinal.height().0 => {}
    //   _ => return Ok(None),
    // }

    // TODO: stuff

    todo!()
  }

  pub(crate) fn list(&self, outpoint: &[u8]) -> Result<Vec<u8>> {
    let outpoint_to_ordinal_ranges = &lmdb::Database::open(
      &self.0,
      Some(OUTPOINT_TO_ORDINAL_RANGES),
      &lmdb::DatabaseOptions::new(lmdb::db::CREATE),
    )
    .unwrap();

    let tx = lmdb::ReadTransaction::new(&self.0).unwrap();

    let access = tx.access();

    let value: &[u8] = access.get(outpoint_to_ordinal_ranges, outpoint).unwrap();

    Ok(value.to_vec())
  }
}

pub(crate) struct WriteTransaction<'a> {
  inner: lmdb::WriteTransaction<'a>,
  env: &'a Environment,
  height_to_hash: lmdb::Database<'a>,
  outpoint_to_ordinal_ranges: lmdb::Database<'a>,
}

impl<'a> WriteTransaction<'a> {
  pub(crate) fn new(environment: &'a Environment) -> Result<Self> {
    let height_to_hash = lmdb::Database::open(
      environment,
      Some(HEIGHT_TO_HASH),
      &lmdb::DatabaseOptions::new(lmdb::db::CREATE),
    )
    .unwrap();

    let outpoint_to_ordinal_ranges = lmdb::Database::open(
      environment,
      Some(OUTPOINT_TO_ORDINAL_RANGES),
      &lmdb::DatabaseOptions::new(lmdb::db::CREATE),
    )
    .unwrap();

    // let outpoint_to_ordinal_ranges =
    //   environment.create_db(Some(), DatabaseFlags::empty())?;

    // let key_to_satpoint = environment.create_db(Some(KEY_TO_SATPOINT), DatabaseFlags::empty())?;

    let tx = lmdb::WriteTransaction::new(environment.clone()).unwrap();

    // Ok(Self {
    //   inner: tx,
    //   height_to_hash,
    //   outpoint_to_ordinal_ranges,
    //   key_to_satpoint,
    // })

    Ok(Self {
      inner: tx,
      env: environment,
      height_to_hash,
      outpoint_to_ordinal_ranges,
    })
  }

  pub(crate) fn abort(self) -> Result {
    Ok(())
  }

  pub(crate) fn commit(self) -> Result {
    Ok(self.inner.commit()?)
  }

  pub(crate) fn height(&self) -> Result<u64> {
    // Ok(
    //   self
    //     .inner
    //     .open_ro_cursor(self.height_to_hash)?
    //     .iter()
    //     .last()
    //     .transpose()?
    //     .map(|(height, _hash)| u64::from_be_bytes(height.try_into().unwrap()) + 1)
    //     .unwrap_or(0),
    // )
    let mut cursor = self.inner.cursor(&self.height_to_hash).unwrap();

    let access = self.inner.access();

    let last: (&[u8], &[u8]) = match cursor.last(&access) {
      Ok(kv) => kv,
      Err(lmdb::Error::Code(-30798)) => return Ok(0),
      Err(error) => return Err(error.into()),
    };

    Ok(u64::from_be_bytes(last.0.try_into().unwrap()) + 1)
  }

  pub(crate) fn outputs_indexed(&self) -> Result<u64> {
    // Ok(
    //   self
    //     .inner
    //     .open_ro_cursor(self.outpoint_to_ordinal_ranges)?
    //     .iter()
    //     .count()
    //     .try_into()?,
    // )
    todo!()
  }

  pub(crate) fn blockhash_at_height(&self, height: u64) -> Result<Option<Vec<u8>>> {
    let access = self.inner.access();

    let value: &[u8] = match access.get(&self.height_to_hash, &height.to_be_bytes()) {
      Ok(value) => value,
      Err(lmdb::Error::Code(-30798)) => return Ok(None),
      Err(error) => return Err(error.into()),
    };

    Ok(Some(value.to_vec()))
  }

  pub(crate) fn set_blockhash_at_height(&mut self, height: u64, blockhash: BlockHash) -> Result {
    let mut write = self.inner.access();

    write
      .put(
        &self.height_to_hash,
        &height.to_be_bytes(),
        blockhash.as_ref(),
        lmdb::put::Flags::empty(),
      )
      .unwrap();

    // Ok(self.inner.put(
    //   self.outpoint_to_ordinal_ranges,
    //   &outpoint,
    //   &ordinal_ranges,
    //   WriteFlags::empty(),
    // )?)

    Ok(())
  }

  pub(crate) fn insert_outpoint(&mut self, outpoint: &[u8], ordinal_ranges: &[u8]) -> Result {
    let mut write = self.inner.access();

    write
      .put(
        &self.outpoint_to_ordinal_ranges,
        outpoint,
        ordinal_ranges,
        lmdb::put::Flags::empty(),
      )
      .unwrap();

    // Ok(self.inner.put(
    //   self.outpoint_to_ordinal_ranges,
    //   &outpoint,
    //   &ordinal_ranges,
    //   WriteFlags::empty(),
    // )?)
    Ok(())
  }

  pub(crate) fn get_ordinal_ranges(&self, outpoint: &[u8]) -> Result<Option<Vec<u8>>> {
    let access = self.inner.access();

    let value: &[u8] = match access.get(&self.outpoint_to_ordinal_ranges, outpoint) {
      Ok(value) => value,
      Err(lmdb::Error::Code(-30798)) => return Ok(None),
      Err(error) => return Err(error.into()),
    };

    Ok(Some(value.to_vec()))
  }

  pub(crate) fn insert_satpoint(&mut self, key: &[u8], satpoint: &[u8]) -> Result {
    // Ok(
    //   self
    //     .inner
    //     .put(self.key_to_satpoint, &key, &satpoint, WriteFlags::empty())?,
    // )
    Ok(())
  }
}
