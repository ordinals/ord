use {
  super::*,
  ord_lmdb_zero::{self as lmdb, EnvBuilder, Environment},
  std::fs,
};

const HEIGHT_TO_HASH: &str = "HEIGHT_TO_HASH";
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

pub(crate) struct Database {
  environment: Arc<Environment>,
  height_to_hash: lmdb::Database<'static>,
  outpoint_to_ordinal_ranges: lmdb::Database<'static>,
}

impl Database {
  pub(crate) fn open(options: &Options) -> Result<Self> {
    let path = "index.lmdb";

    fs::create_dir_all(path)?;

    let environment = unsafe {
      let mut builder = EnvBuilder::new()?;

      builder.set_maxdbs(3)?;
      builder.set_mapsize(options.index_size.0)?;

      Arc::new(builder.open(path, lmdb::open::Flags::empty(), 0o600)?)
    };

    let height_to_hash = lmdb::Database::open(
      environment.clone(),
      Some(HEIGHT_TO_HASH),
      &lmdb::DatabaseOptions::new(lmdb::db::CREATE),
    )?;

    let outpoint_to_ordinal_ranges = lmdb::Database::open(
      environment.clone(),
      Some(OUTPOINT_TO_ORDINAL_RANGES),
      &lmdb::DatabaseOptions::new(lmdb::db::CREATE),
    )?;

    Ok(Self {
      environment,
      height_to_hash,
      outpoint_to_ordinal_ranges,
    })
  }

  pub(crate) fn begin_write(&self) -> Result<WriteTransaction> {
    WriteTransaction::new(self)
  }

  pub(crate) fn print_info(&self) -> Result {
    let stat = self.environment.stat()?;

    let blocks_indexed = self.height()?;

    println!("blocks indexed: {}", blocks_indexed);
    println!(
      "data and metadata: {}",
      ((stat.branch_pages + stat.leaf_pages + stat.overflow_pages) as u64) * stat.psize as u64
    );

    Ok(())
  }

  pub(crate) fn height(&self) -> Result<u64> {
    let tx = lmdb::ReadTransaction::new(self.environment.clone())?;

    let height = tx
      .cursor(&self.height_to_hash)?
      .last::<[u8], [u8]>(&tx.access())
      .into_option()?
      .map(|(key, _value)| u64::from_be_bytes(key.try_into().unwrap()) + 1)
      .unwrap_or_default();

    Ok(height)
  }

  pub(crate) fn list(&self, outpoint: &[u8]) -> Result<Option<Vec<u8>>> {
    Ok(
      lmdb::ReadTransaction::new(self.environment.clone())?
        .access()
        .get::<[u8], [u8]>(&self.outpoint_to_ordinal_ranges, outpoint)
        .into_option()?
        .map(|ranges| ranges.to_vec()),
    )
  }

  pub(crate) fn find(&self, ordinal: Ordinal) -> Result<Option<SatPoint>> {
    let tx = lmdb::ReadTransaction::new(self.environment.clone())?;

    let access = tx.access();

    let mut cursor = tx.cursor(&self.outpoint_to_ordinal_ranges)?;

    while let Some((key, value)) = cursor.next::<[u8], [u8]>(&access).into_option()? {
      let mut offset = 0;
      for chunk in value.chunks_exact(11) {
        let (start, end) = Index::decode_ordinal_range(chunk.try_into().unwrap());
        if start <= ordinal.0 && ordinal.0 < end {
          let outpoint: OutPoint = Decodable::consensus_decode(key)?;
          return Ok(Some(SatPoint {
            outpoint,
            offset: offset + ordinal.0 - start,
          }));
        }
        offset += end - start;
      }
    }

    Ok(None)
  }
}

pub(crate) struct WriteTransaction<'a> {
  height_to_hash: &'a lmdb::Database<'static>,
  lmdb_write_transaction: lmdb::WriteTransaction<'a>,
  outpoint_to_ordinal_ranges: &'a lmdb::Database<'static>,
}

impl<'a> WriteTransaction<'a> {
  pub(crate) fn new(database: &'a Database) -> Result<Self> {
    let lmdb_write_transaction = lmdb::WriteTransaction::new(database.environment.clone())?;

    Ok(Self {
      lmdb_write_transaction,
      height_to_hash: &database.height_to_hash,
      outpoint_to_ordinal_ranges: &database.outpoint_to_ordinal_ranges,
    })
  }

  pub(crate) fn commit(self) -> Result {
    Ok(self.lmdb_write_transaction.commit()?)
  }

  pub(crate) fn height(&self) -> Result<u64> {
    Ok(
      self
        .lmdb_write_transaction
        .cursor(self.height_to_hash)?
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
        .get::<[u8], [u8]>(self.height_to_hash, &height.to_be_bytes())
        .into_option()?
        .map(|value| value.to_vec()),
    )
  }

  pub(crate) fn set_blockhash_at_height(&mut self, height: u64, blockhash: BlockHash) -> Result {
    self.lmdb_write_transaction.access().put(
      self.height_to_hash,
      &height.to_be_bytes(),
      blockhash.as_ref(),
      lmdb::put::Flags::empty(),
    )?;
    Ok(())
  }

  pub(crate) fn insert_outpoint(&mut self, outpoint: &[u8], ordinal_ranges: &[u8]) -> Result {
    self.lmdb_write_transaction.access().put(
      self.outpoint_to_ordinal_ranges,
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
      .del_key(self.outpoint_to_ordinal_ranges, outpoint)?;
    Ok(())
  }

  pub(crate) fn get_ordinal_ranges(&self, outpoint: &[u8]) -> Result<Option<Vec<u8>>> {
    Ok(
      self
        .lmdb_write_transaction
        .access()
        .get::<[u8], [u8]>(self.outpoint_to_ordinal_ranges, outpoint)
        .into_option()?
        .map(|value| value.to_vec()),
    )
  }
}
