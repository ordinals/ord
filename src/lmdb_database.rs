use {
  super::*,
  ord_lmdb_zero::{self as lmdb, EnvBuilder, Environment},
  std::fs,
};

const HEIGHT_TO_HASH: &str = "HEIGHT_TO_HASH";
const KEY_TO_SATPOINT: &str = "KEY_TO_SATPOINT";
const OUTPOINT_TO_ORDINAL_RANGES: &str = "OUTPOINT_TO_ORDINAL_RANGES";

trait LmdbResultExt<T> {
  fn into_option(self) -> Result<Option<T>>;
}

impl<T> LmdbResultExt<T> for lmdb::Result<T> {
  fn into_option(self) -> Result<Option<T>> {
    match self {
      Ok(value) => Ok(Some(value)),
      Err(lmdb::Error::Code(-30798)) => Ok(None),
      Err(error) => Err(error.into()),
    }
  }
}

pub(crate) struct Database(Environment);

impl Database {
  pub(crate) fn open(options: &Options) -> Result<Self> {
    let path = "index.lmdb";

    fs::create_dir_all(path)?;

    let env = unsafe {
      let mut builder = EnvBuilder::new()?;

      builder.set_maxdbs(3)?;
      builder.set_mapsize(options.index_size.0)?;

      builder
        .open(path, lmdb::open::Flags::empty(), 0o600)
        .unwrap()
    };

    Ok(Self(env))
  }

  pub(crate) fn begin_write(&self) -> Result<WriteTransaction> {
    WriteTransaction::new(&self.0)
  }

  pub(crate) fn print_info(&self) -> Result {
    let stat = self.0.stat()?;

    let blocks_indexed = self.height()?;

    println!("blocks indexed: {}", blocks_indexed);
    println!(
      "data and metadata: {}",
      ((stat.branch_pages + stat.leaf_pages + stat.overflow_pages) as u64) * stat.psize as u64
    );

    Ok(())
  }

  pub(crate) fn find(&self, ordinal: Ordinal) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
    let key_to_satpoint = lmdb::Database::open(
      &self.0,
      Some(KEY_TO_SATPOINT),
      &lmdb::DatabaseOptions::new(lmdb::db::CREATE),
    )?;

    let tx = lmdb::ReadTransaction::new(&self.0)?;

    let mut cursor = tx.cursor(key_to_satpoint)?;

    let key = Key::new(ordinal).encode();

    let access = tx.access();
    cursor
      .seek_range_k::<[u8], [u8]>(&access, key.as_slice())
      .into_option()?;

    Ok(
      cursor
        .prev::<[u8], [u8]>(&access)
        .into_option()?
        .map(|(start_key, start_satpoint)| (start_key.to_vec(), start_satpoint.to_vec())),
    )
  }

  pub(crate) fn height(&self) -> Result<u64> {
    let height_to_hash = lmdb::Database::open(
      &self.0,
      Some(HEIGHT_TO_HASH),
      &lmdb::DatabaseOptions::new(lmdb::db::CREATE),
    )?;

    let tx = lmdb::ReadTransaction::new(&self.0)?;

    let height = tx
      .cursor(&height_to_hash)?
      .last::<[u8], [u8]>(&tx.access())
      .into_option()?
      .map(|(key, _value)| u64::from_be_bytes(key.try_into().unwrap()) + 1)
      .unwrap_or_default();

    Ok(height)
  }

  pub(crate) fn list(&self, outpoint: &[u8]) -> Result<Option<Vec<u8>>> {
    let outpoint_to_ordinal_ranges = &lmdb::Database::open(
      &self.0,
      Some(OUTPOINT_TO_ORDINAL_RANGES),
      &lmdb::DatabaseOptions::new(lmdb::db::CREATE),
    )?;

    Ok(
      lmdb::ReadTransaction::new(&self.0)?
        .access()
        .get::<[u8], [u8]>(outpoint_to_ordinal_ranges, outpoint)
        .into_option()?
        .map(|ranges| ranges.to_vec()),
    )
  }
}

pub(crate) struct WriteTransaction<'a> {
  height_to_hash: lmdb::Database<'a>,
  lmdb_write_transaction: lmdb::WriteTransaction<'a>,
  key_to_satpoint: lmdb::Database<'a>,
  outpoint_to_ordinal_ranges: lmdb::Database<'a>,
}

impl<'a> WriteTransaction<'a> {
  pub(crate) fn new(environment: &'a Environment) -> Result<Self> {
    let height_to_hash = lmdb::Database::open(
      environment,
      Some(HEIGHT_TO_HASH),
      &lmdb::DatabaseOptions::new(lmdb::db::CREATE),
    )?;

    let outpoint_to_ordinal_ranges = lmdb::Database::open(
      environment,
      Some(OUTPOINT_TO_ORDINAL_RANGES),
      &lmdb::DatabaseOptions::new(lmdb::db::CREATE),
    )?;

    let key_to_satpoint = lmdb::Database::open(
      environment,
      Some(KEY_TO_SATPOINT),
      &lmdb::DatabaseOptions::new(lmdb::db::CREATE),
    )?;

    let lmdb_write_transaction = lmdb::WriteTransaction::new(environment)?;

    Ok(Self {
      lmdb_write_transaction,
      height_to_hash,
      outpoint_to_ordinal_ranges,
      key_to_satpoint,
    })
  }

  pub(crate) fn commit(self) -> Result {
    Ok(self.lmdb_write_transaction.commit()?)
  }

  pub(crate) fn height(&self) -> Result<u64> {
    Ok(
      self
        .lmdb_write_transaction
        .cursor(&self.height_to_hash)?
        .last::<[u8], [u8]>(&self.lmdb_write_transaction.access())
        .into_option()?
        .map(|(key, _value)| u64::from_be_bytes(key.try_into().unwrap()) + 1)
        .unwrap_or_default(),
    )
  }

  pub(crate) fn blockhash_at_height(&self, height: u64) -> Result<Option<Vec<u8>>> {
    Ok(
      self
        .lmdb_write_transaction
        .access()
        .get::<[u8], [u8]>(&self.height_to_hash, &height.to_be_bytes())
        .into_option()?
        .map(|value| value.to_vec()),
    )
  }

  pub(crate) fn set_blockhash_at_height(&mut self, height: u64, blockhash: BlockHash) -> Result {
    self.lmdb_write_transaction.access().put(
      &self.height_to_hash,
      &height.to_be_bytes(),
      blockhash.as_ref(),
      lmdb::put::Flags::empty(),
    )?;
    Ok(())
  }

  pub(crate) fn insert_outpoint(&mut self, outpoint: &[u8], ordinal_ranges: &[u8]) -> Result {
    self.lmdb_write_transaction.access().put(
      &self.outpoint_to_ordinal_ranges,
      outpoint,
      ordinal_ranges,
      lmdb::put::Flags::empty(),
    )?;
    Ok(())
  }

  pub(crate) fn remove_outpoint(&mut self, outpoint: &[u8]) -> Result {
    self
      .lmdb_write_transaction
      .access()
      .del_key(&self.outpoint_to_ordinal_ranges, outpoint)?;
    Ok(())
  }

  pub(crate) fn get_ordinal_ranges(&self, outpoint: &[u8]) -> Result<Option<Vec<u8>>> {
    Ok(
      self
        .lmdb_write_transaction
        .access()
        .get::<[u8], [u8]>(&self.outpoint_to_ordinal_ranges, outpoint)
        .into_option()?
        .map(|value| value.to_vec()),
    )
  }

  pub(crate) fn insert_satpoint(&mut self, key: &[u8], satpoint: &[u8]) -> Result {
    self.lmdb_write_transaction.access().put(
      &self.key_to_satpoint,
      key,
      satpoint,
      lmdb::put::Flags::empty(),
    )?;
    Ok(())
  }

  pub(crate) fn remove_satpoint(&mut self, key: &[u8]) -> Result {
    let mut cursor = self.lmdb_write_transaction.cursor(&self.key_to_satpoint)?;
    let mut access = self.lmdb_write_transaction.access();
    cursor.seek_range_k::<[u8], [u8]>(&access, key)?;
    cursor.del(&mut access, lmdb::del::Flags::empty())?;
    Ok(())
  }
}
