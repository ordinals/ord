use {
  super::*,
  redb::{ReadableTable, TableDefinition},
};

const HEIGHT_TO_HASH: TableDefinition<'static, u64, [u8]> = TableDefinition::new("HEIGHT_TO_HASH");
const OUTPOINT_TO_ORDINAL_RANGES: TableDefinition<'static, [u8], [u8]> =
  TableDefinition::new("OUTPOINT_TO_ORDINAL_RANGES");
const KEY_TO_SATPOINT: TableDefinition<'static, [u8], [u8]> =
  TableDefinition::new("KEY_TO_SATPOINT");

pub(crate) struct Database(redb::Database);

impl Database {
  pub(crate) fn open(options: &Options) -> Result<Self> {
    match unsafe { redb::Database::open("index.redb") } {
      Ok(database) => Ok(Self(database)),
      Err(redb::Error::Io(error)) if error.kind() == io::ErrorKind::NotFound => unsafe {
        Ok(Self(redb::Database::create(
          "index.redb",
          options.index_size.0,
        )?))
      },
      Err(error) => Err(error.into()),
    }
  }

  pub(crate) fn begin_write(&self) -> Result<WriteTransaction> {
    WriteTransaction::new(&self.0)
  }

  pub(crate) fn print_info(&self) -> Result {
    let tx = self.0.begin_write()?;

    let height_to_hash = tx.open_table(&HEIGHT_TO_HASH)?;

    let blocks_indexed = height_to_hash
      .range(0..)?
      .rev()
      .next()
      .map(|(height, _hash)| height + 1)
      .unwrap_or(0);

    let outputs_indexed = tx.open_table(&OUTPOINT_TO_ORDINAL_RANGES)?.len()?;

    tx.abort()?;

    let stats = self.0.stats()?;

    println!("blocks indexed: {}", blocks_indexed);
    println!("outputs indexed: {}", outputs_indexed);
    println!("tree height: {}", stats.tree_height());
    println!("free pages: {}", stats.free_pages());
    println!("stored: {}", Bytes(stats.stored_bytes()));
    println!("overhead: {}", Bytes(stats.overhead_bytes()));
    println!("fragmented: {}", Bytes(stats.fragmented_bytes()));
    println!(
      "index size: {}",
      Bytes(std::fs::metadata("index.redb")?.len().try_into()?)
    );

    Ok(())
  }

  pub(crate) fn find(&self, ordinal: Ordinal) -> Result<Option<(u64, u64, SatPoint)>> {
    let rtx = self.0.begin_read()?;

    let height_to_hash = match rtx.open_table(&HEIGHT_TO_HASH) {
      Ok(height_to_hash) => height_to_hash,
      Err(redb::Error::TableDoesNotExist(_)) => return Ok(None),
      Err(err) => return Err(err.into()),
    };

    match height_to_hash.range(0..)?.rev().next() {
      Some((height, _hash)) if height >= ordinal.height().0 => {}
      _ => return Ok(None),
    }

    let key_to_satpoint = match rtx.open_table(&KEY_TO_SATPOINT) {
      Ok(key_to_satpoint) => key_to_satpoint,
      Err(redb::Error::TableDoesNotExist(_)) => return Ok(None),
      Err(err) => return Err(err.into()),
    };

    match key_to_satpoint
      .range([].as_slice()..=Key::new(ordinal).encode().as_slice())?
      .rev()
      .next()
    {
      Some((start_key, start_satpoint)) => {
        let start_key = Key::decode(start_key)?;
        let start_satpoint = SatPoint::consensus_decode(start_satpoint)?;
        Ok(Some((
          start_key.block,
          start_key.transaction,
          SatPoint {
            offset: start_satpoint.offset + (ordinal.0 - start_key.ordinal),
            outpoint: start_satpoint.outpoint,
          },
        )))
      }
      None => Ok(None),
    }
  }

  pub(crate) fn list(&self, outpoint: &[u8]) -> Result<Vec<u8>> {
    Ok(
      self
        .0
        .begin_read()?
        .open_table(&OUTPOINT_TO_ORDINAL_RANGES)?
        .get(outpoint)?
        .ok_or("Could not find outpoint in index")?
        .to_vec(),
    )
  }
}

pub(crate) struct WriteTransaction<'a> {
  inner: redb::DatabaseTransaction<'a>,
  height_to_hash: redb::Table<'a, u64, [u8]>,
  outpoint_to_ordinal_ranges: redb::Table<'a, [u8], [u8]>,
  key_to_satpoint: redb::Table<'a, [u8], [u8]>,
}

impl<'a> WriteTransaction<'a> {
  pub(crate) fn new(database: &'a redb::Database) -> Result<Self> {
    let inner = database.begin_write()?;
    let height_to_hash = inner.open_table(&HEIGHT_TO_HASH)?;
    let outpoint_to_ordinal_ranges = inner.open_table(&OUTPOINT_TO_ORDINAL_RANGES)?;
    let key_to_satpoint = inner.open_table(&KEY_TO_SATPOINT)?;

    Ok(Self {
      inner,
      height_to_hash,
      outpoint_to_ordinal_ranges,
      key_to_satpoint,
    })
  }

  pub(crate) fn abort(self) -> Result {
    self.inner.abort()?;
    Ok(())
  }

  pub(crate) fn commit(self) -> Result {
    self.inner.commit()?;
    Ok(())
  }

  pub(crate) fn height(&self) -> Result<u64> {
    Ok(
      self
        .height_to_hash
        .range(0..)?
        .rev()
        .next()
        .map(|(height, _hash)| height + 1)
        .unwrap_or(0),
    )
  }

  pub(crate) fn blockhash_at_height(&self, height: u64) -> Result<Option<&[u8]>> {
    Ok(self.height_to_hash.get(&height)?)
  }

  pub(crate) fn set_blockhash_at_height(&mut self, height: u64, blockhash: BlockHash) -> Result {
    self.height_to_hash.insert(&height, &blockhash)?;
    Ok(())
  }

  pub(crate) fn insert_outpoint(&mut self, outpoint: &[u8], ordinal_ranges: &[u8]) -> Result {
    self
      .outpoint_to_ordinal_ranges
      .insert(&outpoint, &ordinal_ranges)?;
    Ok(())
  }

  pub(crate) fn get_ordinal_ranges(&self, outpoint: &[u8]) -> Result<Option<&[u8]>> {
    Ok(self.outpoint_to_ordinal_ranges.get(&outpoint)?)
  }

  pub(crate) fn insert_satpoint(&mut self, key: &[u8], satpoint: &[u8]) -> Result {
    self.key_to_satpoint.insert(key, satpoint)?;
    Ok(())
  }
}
