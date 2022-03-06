use super::*;

use lmdb::{
  Cursor, Database as Table, DatabaseFlags, Environment, Error, RwTransaction, Transaction,
  WriteFlags,
};

use std::{fs, path::Path};

const HEIGHT_TO_HASH: &'static str = "HEIGHT_TO_HASH";
const OUTPOINT_TO_ORDINAL_RANGES: &'static str = "OUTPOINT_TO_ORDINAL_RANGES";
const KEY_TO_SATPOINT: &'static str = "KEY_TO_SATPOINT";

pub(crate) struct Database(Environment);

impl Database {
  pub(crate) fn open(options: &Options) -> Result<Self> {
    let path = Path::new("index.lmdb");

    let result = Environment::new()
      .set_map_size(options.index_size.0)
      .set_max_dbs(3)
      .open(path);

    match result {
      Ok(env) => Ok(Self(env)),
      Err(Error::Other(_)) => {
        fs::create_dir_all(path)?;
        Ok(Self(
          Environment::new()
            .set_map_size(options.index_size.0)
            .set_max_dbs(3)
            .open(path)?,
        ))
      }
      Err(error) => Err(error.into()),
    }
  }

  pub(crate) fn begin_write(&self) -> Result<WriteTransaction> {
    Ok(WriteTransaction::new(&self.0)?)
  }

  pub(crate) fn print_info(&self) -> Result {
    let tx = self.begin_write()?;

    let blocks_indexed = tx.height()?;

    let outputs_indexed = tx.outputs_indexed()?;

    tx.abort()?;

    let stats = self.0.stat()?;

    println!("blocks indexed: {}", blocks_indexed);
    println!("outputs indexed: {}", outputs_indexed);
    println!("tree height: {}", stats.depth());
    // println!("free pages: {}", stats.free_pages());
    println!(
      "stored: {}",
      Bytes(
        (stats.branch_pages() + stats.leaf_pages() + stats.overflow_pages())
          * stats.page_size() as usize
      )
    );
    // println!("overhead: {}", Bytes(stats.overhead_bytes()));
    // println!("fragmented: {}", Bytes(stats.fragmented_bytes()));
    println!(
      "index size: {}",
      Bytes(std::fs::metadata("index.lmdb")?.len().try_into()?)
    );

    Ok(())
  }

  pub(crate) fn find(&self, ordinal: Ordinal) -> Result<Option<(u64, u64, SatPoint)>> {
    let height_to_hash = self.0.open_db(Some(HEIGHT_TO_HASH))?;

    let _key_to_satpoint = self.0.open_db(Some(KEY_TO_SATPOINT))?;

    let rtx = self.0.begin_ro_txn()?;

    match rtx.open_ro_cursor(height_to_hash)?.iter().last() {
      Some(result) if u64::from_be_bytes(result?.0.try_into()?) >= ordinal.height().0 => {}
      _ => return Ok(None),
    }

    // TODO: stuff

    todo!()
  }

  pub(crate) fn list(&self, outpoint: &[u8]) -> Result<Vec<u8>> {
    Ok(
      self
        .0
        .begin_ro_txn()?
        .get(self.0.open_db(Some(OUTPOINT_TO_ORDINAL_RANGES))?, &outpoint)?
        .to_vec(),
    )
  }
}

pub(crate) struct WriteTransaction<'a> {
  inner: RwTransaction<'a>,
  height_to_hash: Table,
  outpoint_to_ordinal_ranges: Table,
  key_to_satpoint: Table,
}

impl<'a> WriteTransaction<'a> {
  pub(crate) fn new(environment: &'a Environment) -> Result<Self> {
    let height_to_hash = environment.create_db(Some(HEIGHT_TO_HASH), DatabaseFlags::empty())?;

    let outpoint_to_ordinal_ranges =
      environment.create_db(Some(OUTPOINT_TO_ORDINAL_RANGES), DatabaseFlags::empty())?;

    let key_to_satpoint = environment.create_db(Some(KEY_TO_SATPOINT), DatabaseFlags::empty())?;

    let tx = environment.begin_rw_txn()?;

    Ok(Self {
      inner: tx,
      height_to_hash,
      outpoint_to_ordinal_ranges,
      key_to_satpoint,
    })
  }

  pub(crate) fn abort(self) -> Result {
    Ok(self.inner.abort())
  }

  pub(crate) fn commit(self) -> Result {
    Ok(self.inner.commit()?)
  }

  pub(crate) fn height(&self) -> Result<u64> {
    Ok(
      self
        .inner
        .open_ro_cursor(self.height_to_hash)?
        .iter()
        .last()
        .transpose()?
        .map(|(height, _hash)| u64::from_be_bytes(height.try_into().unwrap()) + 1)
        .unwrap_or(0),
    )
  }

  pub(crate) fn outputs_indexed(&self) -> Result<u64> {
    Ok(
      self
        .inner
        .open_ro_cursor(self.outpoint_to_ordinal_ranges)?
        .iter()
        .count()
        .try_into()?,
    )
  }

  pub(crate) fn blockhash_at_height(&self, height: u64) -> Result<Option<&[u8]>> {
    Ok(
      self
        .inner
        .get(self.height_to_hash, &height.to_be_bytes())
        .ok(),
    )
  }

  pub(crate) fn set_blockhash_at_height(&mut self, height: u64, blockhash: BlockHash) -> Result {
    Ok(self.inner.put(
      self.height_to_hash,
      &height.to_be_bytes(),
      &blockhash,
      WriteFlags::empty(),
    )?)
  }

  pub(crate) fn insert_outpoint(&mut self, outpoint: &[u8], ordinal_ranges: &[u8]) -> Result {
    Ok(self.inner.put(
      self.outpoint_to_ordinal_ranges,
      &outpoint,
      &ordinal_ranges,
      WriteFlags::empty(),
    )?)
  }

  pub(crate) fn get_ordinal_ranges(&self, outpoint: &[u8]) -> Result<Option<&[u8]>> {
    Ok(
      self
        .inner
        .get(self.outpoint_to_ordinal_ranges, &outpoint)
        .ok(),
    )
  }

  pub(crate) fn insert_satpoint(&mut self, key: &[u8], satpoint: &[u8]) -> Result {
    Ok(
      self
        .inner
        .put(self.key_to_satpoint, &key, &satpoint, WriteFlags::empty())?,
    )
  }
}
