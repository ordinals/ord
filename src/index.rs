use {
  self::{
    entry::{
      BlockHashValue, Entry, InscriptionEntry, InscriptionEntryValue, InscriptionIdValue,
      OutPointValue, SatPointValue, SatRange,
    },
    updater::Updater,
  },
  super::*,
  crate::wallet::Wallet,
  bitcoin::BlockHeader,
  bitcoincore_rpc::{json::GetBlockHeaderResult, Client},
  chrono::SubsecRound,
  indicatif::{ProgressBar, ProgressStyle},
  log::log_enabled,
  redb::{
    Database, MultimapTable, MultimapTableDefinition, ReadableMultimapTable, ReadableTable, Table,
    TableDefinition, WriteTransaction,
  },
  std::collections::HashMap,
  std::io::{BufWriter, Write},
  std::sync::atomic::{self, AtomicBool},
};

mod entry;
mod fetcher;
mod rtx;
mod updater;

const SCHEMA_VERSION: u64 = 5;

macro_rules! define_table {
  ($name:ident, $key:ty, $value:ty) => {
    const $name: TableDefinition<$key, $value> = TableDefinition::new(stringify!($name));
  };
}

macro_rules! define_multimap_table {
  ($name:ident, $key:ty, $value:ty) => {
    const $name: MultimapTableDefinition<$key, $value> =
      MultimapTableDefinition::new(stringify!($name));
  };
}

define_table! { HEIGHT_TO_BLOCK_HASH, u64, &BlockHashValue }
define_table! { INSCRIPTION_ID_TO_INSCRIPTION_ENTRY, &InscriptionIdValue, InscriptionEntryValue }
define_table! { INSCRIPTION_ID_TO_SATPOINT, &InscriptionIdValue, &SatPointValue }
define_table! { INSCRIPTION_NUMBER_TO_INSCRIPTION_ID, i64, &InscriptionIdValue }
define_table! { OUTPOINT_TO_SAT_RANGES, &OutPointValue, &[u8] }
define_table! { OUTPOINT_TO_VALUE, &OutPointValue, u64}
define_table! { REINSCRIPTION_ID_TO_SEQUENCE_NUMBER, &InscriptionIdValue, u64 }
define_multimap_table! { SATPOINT_TO_INSCRIPTION_ID, &SatPointValue, &InscriptionIdValue }
define_multimap_table! { SAT_TO_INSCRIPTION_ID, u64, &InscriptionIdValue }
define_table! { SAT_TO_SATPOINT, u64, &SatPointValue }
define_table! { STATISTIC_TO_COUNT, u64, u64 }
define_table! { WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP, u64, u128 }

pub(crate) struct Index {
  client: Client,
  database: Database,
  path: PathBuf,
  first_inscription_height: u64,
  genesis_block_coinbase_transaction: Transaction,
  genesis_block_coinbase_txid: Txid,
  height_limit: Option<u64>,
  options: Options,
  reorged: AtomicBool,
}

#[derive(Debug, PartialEq)]
pub(crate) enum List {
  Spent,
  Unspent(Vec<(u64, u64)>),
}

#[derive(Copy, Clone)]
#[repr(u64)]
pub(crate) enum Statistic {
  Schema = 0,
  Commits = 1,
  LostSats = 2,
  OutputsTraversed = 3,
  SatRanges = 4,
  UnboundInscriptions = 5,
}

impl Statistic {
  fn key(self) -> u64 {
    self.into()
  }
}

impl From<Statistic> for u64 {
  fn from(statistic: Statistic) -> Self {
    statistic as u64
  }
}

#[derive(Serialize)]
pub(crate) struct Info {
  pub(crate) blocks_indexed: u64,
  pub(crate) branch_pages: u64,
  pub(crate) fragmented_bytes: u64,
  pub(crate) index_file_size: u64,
  pub(crate) index_path: PathBuf,
  pub(crate) leaf_pages: u64,
  pub(crate) metadata_bytes: u64,
  pub(crate) outputs_traversed: u64,
  pub(crate) page_size: usize,
  pub(crate) sat_ranges: u64,
  pub(crate) stored_bytes: u64,
  pub(crate) transactions: Vec<TransactionInfo>,
  pub(crate) tree_height: u32,
  pub(crate) utxos_indexed: u64,
}

#[derive(Serialize)]
pub(crate) struct TransactionInfo {
  pub(crate) starting_block_count: u64,
  pub(crate) starting_timestamp: u128,
}

trait BitcoinCoreRpcResultExt<T> {
  fn into_option(self) -> Result<Option<T>>;
}

impl<T> BitcoinCoreRpcResultExt<T> for Result<T, bitcoincore_rpc::Error> {
  fn into_option(self) -> Result<Option<T>> {
    match self {
      Ok(ok) => Ok(Some(ok)),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { code: -8, .. },
      ))) => Ok(None),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { message, .. },
      )))
        if message.ends_with("not found") =>
      {
        Ok(None)
      }
      Err(err) => Err(err.into()),
    }
  }
}

impl Index {
  pub(crate) fn open(options: &Options) -> Result<Self> {
    let client = options.bitcoin_rpc_client()?;

    let path = if let Some(path) = &options.index {
      path.clone()
    } else {
      options.data_dir()?.join("index.redb")
    };

    if let Err(err) = fs::create_dir_all(path.parent().unwrap()) {
      bail!(
        "failed to create data dir `{}`: {err}",
        path.parent().unwrap().display()
      );
    }

    let db_cache_size = match options.db_cache_size {
      Some(db_cache_size) => db_cache_size,
      None => {
        let mut sys = System::new();
        sys.refresh_memory();
        usize::try_from(sys.total_memory() / 4)?
      }
    };

    log::info!("Setting DB cache size to {} bytes", db_cache_size);

    let database = match Database::builder()
      .set_cache_size(db_cache_size)
      .open(&path)
    {
      Ok(database) => {
        let schema_version = database
          .begin_read()?
          .open_table(STATISTIC_TO_COUNT)?
          .get(&Statistic::Schema.key())?
          .map(|x| x.value())
          .unwrap_or(0);

        match schema_version.cmp(&SCHEMA_VERSION) {
          cmp::Ordering::Less =>
            bail!(
              "index at `{}` appears to have been built with an older, incompatible version of ord, consider deleting and rebuilding the index: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
              path.display()
            ),
          cmp::Ordering::Greater =>
            bail!(
              "index at `{}` appears to have been built with a newer, incompatible version of ord, consider updating ord: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
              path.display()
            ),
          cmp::Ordering::Equal => {
          }
        }

        database
      }
      Err(_) => {
        let database = Database::builder()
          .set_cache_size(db_cache_size)
          .create(&path)?;

        let mut tx = database.begin_write()?;

        if cfg!(test) {
          tx.set_durability(redb::Durability::None);
        } else {
          tx.set_durability(redb::Durability::Immediate);
        };

        tx.open_table(HEIGHT_TO_BLOCK_HASH)?;
        tx.open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY)?;
        tx.open_table(INSCRIPTION_ID_TO_SATPOINT)?;
        tx.open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?;
        tx.open_table(OUTPOINT_TO_VALUE)?;
        tx.open_table(REINSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;
        tx.open_multimap_table(SATPOINT_TO_INSCRIPTION_ID)?;
        tx.open_multimap_table(SAT_TO_INSCRIPTION_ID)?;
        tx.open_table(SAT_TO_SATPOINT)?;
        tx.open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?;

        tx.open_table(STATISTIC_TO_COUNT)?
          .insert(&Statistic::Schema.key(), &SCHEMA_VERSION)?;

        if options.index_sats {
          tx.open_table(OUTPOINT_TO_SAT_RANGES)?
            .insert(&OutPoint::null().store(), [].as_slice())?;
        }

        tx.commit()?;

        database
      }
    };

    let genesis_block_coinbase_transaction =
      options.chain().genesis_block().coinbase().unwrap().clone();

    Ok(Self {
      genesis_block_coinbase_txid: genesis_block_coinbase_transaction.txid(),
      client,
      database,
      path,
      first_inscription_height: options.first_inscription_height(),
      genesis_block_coinbase_transaction,
      height_limit: options.height_limit,
      reorged: AtomicBool::new(false),
      options: options.clone(),
    })
  }

  pub(crate) fn get_unspent_outputs(&self, _wallet: Wallet) -> Result<BTreeMap<OutPoint, Amount>> {
    let mut utxos = BTreeMap::new();
    utxos.extend(
      self
        .client
        .list_unspent(None, None, None, None, None)?
        .into_iter()
        .map(|utxo| {
          let outpoint = OutPoint::new(utxo.txid, utxo.vout);
          let amount = utxo.amount;

          (outpoint, amount)
        }),
    );

    #[derive(Deserialize)]
    pub(crate) struct JsonOutPoint {
      txid: bitcoin::Txid,
      vout: u32,
    }

    for JsonOutPoint { txid, vout } in self
      .client
      .call::<Vec<JsonOutPoint>>("listlockunspent", &[])?
    {
      utxos.insert(
        OutPoint { txid, vout },
        Amount::from_sat(self.client.get_raw_transaction(&txid, None)?.output[vout as usize].value),
      );
    }
    let rtx = self.database.begin_read()?;
    let outpoint_to_value = rtx.open_table(OUTPOINT_TO_VALUE)?;
    for outpoint in utxos.keys() {
      if outpoint_to_value.get(&outpoint.store())?.is_none() {
        return Err(anyhow!(
          "output in Bitcoin Core wallet but not in ord index: {outpoint}"
        ));
      }
    }

    Ok(utxos)
  }

  pub(crate) fn get_unspent_output_ranges(
    &self,
    wallet: Wallet,
  ) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
    self
      .get_unspent_outputs(wallet)?
      .into_keys()
      .map(|outpoint| match self.list(outpoint)? {
        Some(List::Unspent(sat_ranges)) => Ok((outpoint, sat_ranges)),
        Some(List::Spent) => bail!("output {outpoint} in wallet but is spent according to index"),
        None => bail!("index has not seen {outpoint}"),
      })
      .collect()
  }

  pub(crate) fn has_sat_index(&self) -> Result<bool> {
    match self.begin_read()?.0.open_table(OUTPOINT_TO_SAT_RANGES) {
      Ok(_) => Ok(true),
      Err(redb::TableError::TableDoesNotExist(_)) => Ok(false),
      Err(err) => Err(err.into()),
    }
  }

  fn require_sat_index(&self, feature: &str) -> Result {
    if !self.has_sat_index()? {
      bail!("{feature} requires index created with `--index-sats` flag")
    }

    Ok(())
  }

  pub(crate) fn info(&self) -> Result<Info> {
    let wtx = self.begin_write()?;

    let stats = wtx.stats()?;

    let info = {
      let statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;
      let sat_ranges = statistic_to_count
        .get(&Statistic::SatRanges.key())?
        .map(|x| x.value())
        .unwrap_or(0);
      let outputs_traversed = statistic_to_count
        .get(&Statistic::OutputsTraversed.key())?
        .map(|x| x.value())
        .unwrap_or(0);
      Info {
        index_path: self.path.clone(),
        blocks_indexed: wtx
          .open_table(HEIGHT_TO_BLOCK_HASH)?
          .range(0..)?
          .next_back()
          .and_then(|result| result.ok())
          .map(|(height, _hash)| height.value() + 1)
          .unwrap_or(0),
        branch_pages: stats.branch_pages(),
        fragmented_bytes: stats.fragmented_bytes(),
        index_file_size: fs::metadata(&self.path)?.len(),
        leaf_pages: stats.leaf_pages(),
        metadata_bytes: stats.metadata_bytes(),
        sat_ranges,
        outputs_traversed,
        page_size: stats.page_size(),
        stored_bytes: stats.stored_bytes(),
        transactions: wtx
          .open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?
          .range(0..)?
          .flat_map(|result| {
            result.map(
              |(starting_block_count, starting_timestamp)| TransactionInfo {
                starting_block_count: starting_block_count.value(),
                starting_timestamp: starting_timestamp.value(),
              },
            )
          })
          .collect(),
        tree_height: stats.tree_height(),
        utxos_indexed: wtx.open_table(OUTPOINT_TO_SAT_RANGES)?.len()?,
      }
    };

    Ok(info)
  }

  pub(crate) fn update(&self) -> Result {
    Updater::update(self)
  }

  pub(crate) fn export(&self, filename: &String) -> Result {
    let mut writer = BufWriter::new(File::create(filename)?);

    for result in self
      .database
      .begin_read()?
      .open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?
      .iter()?
    {
      let (number, id) = result?;
      writeln!(
        writer,
        "{}\t{}",
        number.value(),
        InscriptionId::load(*id.value())
      )?;
    }

    writer.flush()?;
    Ok(())
  }

  pub(crate) fn is_reorged(&self) -> bool {
    self.reorged.load(atomic::Ordering::Relaxed)
  }

  fn begin_read(&self) -> Result<rtx::Rtx> {
    Ok(rtx::Rtx(self.database.begin_read()?))
  }

  fn begin_write(&self) -> Result<WriteTransaction> {
    if cfg!(test) {
      let mut tx = self.database.begin_write()?;
      tx.set_durability(redb::Durability::None);
      Ok(tx)
    } else {
      Ok(self.database.begin_write()?)
    }
  }

  fn increment_statistic(wtx: &WriteTransaction, statistic: Statistic, n: u64) -> Result {
    let mut statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;
    let value = statistic_to_count
      .get(&(statistic.key()))?
      .map(|x| x.value())
      .unwrap_or(0)
      + n;
    statistic_to_count.insert(&statistic.key(), &value)?;
    Ok(())
  }

  #[cfg(test)]
  pub(crate) fn statistic(&self, statistic: Statistic) -> u64 {
    self
      .database
      .begin_read()
      .unwrap()
      .open_table(STATISTIC_TO_COUNT)
      .unwrap()
      .get(&statistic.key())
      .unwrap()
      .map(|x| x.value())
      .unwrap_or(0)
  }

  pub(crate) fn block_count(&self) -> Result<u64> {
    self.begin_read()?.block_count()
  }

  pub(crate) fn block_height(&self) -> Result<Option<Height>> {
    self.begin_read()?.block_height()
  }

  pub(crate) fn block_hash(&self, height: Option<u64>) -> Result<Option<BlockHash>> {
    self.begin_read()?.block_hash(height)
  }

  pub(crate) fn blocks(&self, take: usize) -> Result<Vec<(u64, BlockHash)>> {
    let mut blocks = Vec::new();

    let rtx = self.begin_read()?;

    let block_count = rtx.block_count()?;

    let height_to_block_hash = rtx.0.open_table(HEIGHT_TO_BLOCK_HASH)?;

    for next in height_to_block_hash.range(0..block_count)?.rev().take(take) {
      let next = next?;
      blocks.push((next.0.value(), Entry::load(*next.1.value())));
    }

    Ok(blocks)
  }

  pub(crate) fn rare_sat_satpoints(&self) -> Result<Option<Vec<(Sat, SatPoint)>>> {
    if self.has_sat_index()? {
      let mut result = Vec::new();

      let rtx = self.database.begin_read()?;

      let sat_to_satpoint = rtx.open_table(SAT_TO_SATPOINT)?;

      for range in sat_to_satpoint.range(0..)? {
        let (sat, satpoint) = range?;
        result.push((Sat(sat.value()), Entry::load(*satpoint.value())));
      }

      Ok(Some(result))
    } else {
      Ok(None)
    }
  }

  pub(crate) fn rare_sat_satpoint(&self, sat: Sat) -> Result<Option<SatPoint>> {
    if self.has_sat_index()? {
      Ok(
        self
          .database
          .begin_read()?
          .open_table(SAT_TO_SATPOINT)?
          .get(&sat.n())?
          .map(|satpoint| Entry::load(*satpoint.value())),
      )
    } else {
      Ok(None)
    }
  }

  pub(crate) fn block_header(&self, hash: BlockHash) -> Result<Option<BlockHeader>> {
    self.client.get_block_header(&hash).into_option()
  }

  pub(crate) fn block_header_info(&self, hash: BlockHash) -> Result<Option<GetBlockHeaderResult>> {
    self.client.get_block_header_info(&hash).into_option()
  }

  pub(crate) fn get_block_by_height(&self, height: u64) -> Result<Option<Block>> {
    Ok(
      self
        .client
        .get_block_hash(height)
        .into_option()?
        .map(|hash| self.client.get_block(&hash))
        .transpose()?,
    )
  }

  pub(crate) fn get_block_by_hash(&self, hash: BlockHash) -> Result<Option<Block>> {
    self.client.get_block(&hash).into_option()
  }

  pub(crate) fn get_inscription_id_by_sat(&self, sat: Sat) -> Result<Option<InscriptionId>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_multimap_table(SAT_TO_INSCRIPTION_ID)?
        .get(&sat.n())?
        .next()
        .and_then(|result| result.ok())
        .map(|inscription_id| Entry::load(*inscription_id.value())),
    )
  }

  pub(crate) fn get_inscription_id_by_inscription_number(
    &self,
    n: i64,
  ) -> Result<Option<InscriptionId>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?
        .get(&n)?
        .map(|id| Entry::load(*id.value())),
    )
  }

  pub(crate) fn get_inscription_satpoint_by_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<SatPoint>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_ID_TO_SATPOINT)?
        .get(&inscription_id.store())?
        .map(|satpoint| Entry::load(*satpoint.value())),
    )
  }

  pub(crate) fn get_inscription_by_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<Inscription>> {
    if self
      .database
      .begin_read()?
      .open_table(INSCRIPTION_ID_TO_SATPOINT)?
      .get(&inscription_id.store())?
      .is_none()
    {
      return Ok(None);
    }

    Ok(self.get_transaction(inscription_id.txid)?.and_then(|tx| {
      Inscription::from_transaction(&tx)
        .get(inscription_id.index as usize)
        .map(|transaction_inscription| transaction_inscription.inscription.clone())
    }))
  }

  pub(crate) fn get_inscriptions_on_output_with_satpoints(
    &self,
    outpoint: OutPoint,
  ) -> Result<Vec<(SatPoint, InscriptionId)>> {
    let rtx = &self.database.begin_read()?;
    let sat_to_id = rtx.open_multimap_table(SATPOINT_TO_INSCRIPTION_ID)?;
    let re_id_to_seq_num = rtx.open_table(REINSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;

    Self::inscriptions_on_output_ordered(&re_id_to_seq_num, &sat_to_id, outpoint)
  }

  pub(crate) fn get_inscriptions_on_output(
    &self,
    outpoint: OutPoint,
  ) -> Result<Vec<InscriptionId>> {
    Ok(
      self
        .get_inscriptions_on_output_with_satpoints(outpoint)?
        .iter()
        .map(|(_satpoint, inscription_id)| *inscription_id)
        .collect(),
    )
  }

  pub(crate) fn get_transaction(&self, txid: Txid) -> Result<Option<Transaction>> {
    if txid == self.genesis_block_coinbase_txid {
      Ok(Some(self.genesis_block_coinbase_transaction.clone()))
    } else {
      self.client.get_raw_transaction(&txid, None).into_option()
    }
  }

  pub(crate) fn get_transaction_blockhash(&self, txid: Txid) -> Result<Option<BlockHash>> {
    Ok(
      self
        .client
        .get_raw_transaction_info(&txid, None)
        .into_option()?
        .and_then(|info| {
          if info.in_active_chain.unwrap_or_default() {
            info.blockhash
          } else {
            None
          }
        }),
    )
  }

  pub(crate) fn is_transaction_in_active_chain(&self, txid: Txid) -> Result<bool> {
    Ok(
      self
        .client
        .get_raw_transaction_info(&txid, None)
        .into_option()?
        .and_then(|info| info.in_active_chain)
        .unwrap_or(false),
    )
  }

  pub(crate) fn find(&self, sat: u64) -> Result<Option<SatPoint>> {
    self.require_sat_index("find")?;

    let rtx = self.begin_read()?;

    if rtx.block_count()? <= Sat(sat).height().n() {
      return Ok(None);
    }

    let outpoint_to_sat_ranges = rtx.0.open_table(OUTPOINT_TO_SAT_RANGES)?;

    for range in outpoint_to_sat_ranges.range::<&[u8; 36]>(&[0; 36]..)? {
      let (key, value) = range?;
      let mut offset = 0;
      for chunk in value.value().chunks_exact(11) {
        let (start, end) = SatRange::load(chunk.try_into().unwrap());
        if start <= sat && sat < end {
          return Ok(Some(SatPoint {
            outpoint: Entry::load(*key.value()),
            offset: offset + sat - start,
          }));
        }
        offset += end - start;
      }
    }

    Ok(None)
  }

  fn list_inner(&self, outpoint: OutPointValue) -> Result<Option<Vec<u8>>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(OUTPOINT_TO_SAT_RANGES)?
        .get(&outpoint)?
        .map(|outpoint| outpoint.value().to_vec()),
    )
  }

  pub(crate) fn list(&self, outpoint: OutPoint) -> Result<Option<List>> {
    self.require_sat_index("list")?;

    let array = outpoint.store();

    let sat_ranges = self.list_inner(array)?;

    match sat_ranges {
      Some(sat_ranges) => Ok(Some(List::Unspent(
        sat_ranges
          .chunks_exact(11)
          .map(|chunk| SatRange::load(chunk.try_into().unwrap()))
          .collect(),
      ))),
      None => {
        if self.is_transaction_in_active_chain(outpoint.txid)? {
          Ok(Some(List::Spent))
        } else {
          Ok(None)
        }
      }
    }
  }

  pub(crate) fn block_time(&self, height: Height) -> Result<Blocktime> {
    let height = height.n();

    match self.get_block_by_height(height)? {
      Some(block) => Ok(Blocktime::confirmed(block.header.time)),
      None => {
        let tx = self.database.begin_read()?;

        let current = tx
          .open_table(HEIGHT_TO_BLOCK_HASH)?
          .range(0..)?
          .next_back()
          .and_then(|result| result.ok())
          .map(|(height, _hash)| height)
          .map(|x| x.value())
          .unwrap_or(0);

        let expected_blocks = height.checked_sub(current).with_context(|| {
          format!("current {current} height is greater than sat height {height}")
        })?;

        Ok(Blocktime::Expected(
          Utc::now()
            .round_subsecs(0)
            .checked_add_signed(chrono::Duration::seconds(
              10 * 60 * i64::try_from(expected_blocks)?,
            ))
            .ok_or_else(|| anyhow!("block timestamp out of range"))?,
        ))
      }
    }
  }

  pub(crate) fn get_inscriptions(
    &self,
    n: Option<usize>,
  ) -> Result<BTreeMap<SatPoint, InscriptionId>> {
    let rtx = self.database.begin_read()?;

    let mut result = BTreeMap::new();

    for range_result in rtx
      .open_multimap_table(SATPOINT_TO_INSCRIPTION_ID)?
      .range::<&[u8; 44]>(&[0; 44]..)?
    {
      let (satpoint, ids) = range_result?;
      for id_result in ids {
        let id = id_result?;
        result.insert(Entry::load(*satpoint.value()), Entry::load(*id.value()));
      }
      if result.len() >= n.unwrap_or(usize::MAX) {
        break;
      }
    }

    Ok(result)
  }

  pub(crate) fn get_homepage_inscriptions(&self) -> Result<Vec<InscriptionId>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?
        .iter()?
        .rev()
        .take(8)
        .flat_map(|result| result.map(|(_number, id)| Entry::load(*id.value())))
        .collect(),
    )
  }

  pub(crate) fn get_latest_inscriptions_with_prev_and_next(
    &self,
    n: usize,
    from: Option<i64>,
  ) -> Result<(Vec<InscriptionId>, Option<i64>, Option<i64>)> {
    let rtx = self.database.begin_read()?;

    let inscription_number_to_inscription_id =
      rtx.open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?;

    let latest = match inscription_number_to_inscription_id.iter()?.next_back() {
      Some(Ok((number, _id))) => number.value(),
      Some(Err(_)) => return Ok(Default::default()),
      None => return Ok(Default::default()),
    };

    let from = from.unwrap_or(latest);

    let prev = if let Some(prev) = from.checked_sub(n.try_into()?) {
      inscription_number_to_inscription_id
        .get(&prev)?
        .map(|_| prev)
    } else {
      None
    };

    let next = if from < latest {
      Some(
        from
          .checked_add(n.try_into()?)
          .unwrap_or(latest)
          .min(latest),
      )
    } else {
      None
    };

    let inscriptions = inscription_number_to_inscription_id
      .range(..=from)?
      .rev()
      .take(n)
      .flat_map(|result| result.map(|(_number, id)| Entry::load(*id.value())))
      .collect();

    Ok((inscriptions, prev, next))
  }

  pub(crate) fn get_feed_inscriptions(&self, n: usize) -> Result<Vec<(i64, InscriptionId)>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_NUMBER_TO_INSCRIPTION_ID)?
        .iter()?
        .rev()
        .take(n)
        .flat_map(|result| result.map(|(number, id)| (number.value(), Entry::load(*id.value()))))
        .collect(),
    )
  }

  pub(crate) fn get_inscription_entry(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<InscriptionEntry>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY)?
        .get(&inscription_id.store())?
        .map(|value| InscriptionEntry::load(value.value())),
    )
  }

  #[cfg(test)]
  fn assert_inscription_location(
    &self,
    inscription_id: InscriptionId,
    satpoint: SatPoint,
    sat: Option<u64>,
  ) {
    let rtx = self.database.begin_read().unwrap();

    let satpoint_to_inscription_id = rtx.open_multimap_table(SATPOINT_TO_INSCRIPTION_ID).unwrap();

    let inscription_id_to_satpoint = rtx.open_table(INSCRIPTION_ID_TO_SATPOINT).unwrap();

    assert_eq!(
      satpoint_to_inscription_id.len().unwrap(),
      inscription_id_to_satpoint.len().unwrap(),
    );

    assert_eq!(
      SatPoint::load(
        *inscription_id_to_satpoint
          .get(&inscription_id.store())
          .unwrap()
          .unwrap()
          .value()
      ),
      satpoint,
    );

    assert!(satpoint_to_inscription_id
      .get(&satpoint.store())
      .unwrap()
      .any(|id| InscriptionId::load(*id.unwrap().value()) == inscription_id));

    match sat {
      Some(sat) => {
        if self.has_sat_index().unwrap() {
          // unbound inscriptions should not be assigned to a sat
          assert!(satpoint.outpoint != unbound_outpoint());
          assert!(rtx
            .open_multimap_table(SAT_TO_INSCRIPTION_ID)
            .unwrap()
            .get(&sat)
            .unwrap()
            .any(|id| InscriptionId::load(*id.unwrap().value()) == inscription_id));

          // we do not track common sats (only the sat ranges)
          if !Sat(sat).is_common() {
            assert_eq!(
              SatPoint::load(
                *rtx
                  .open_table(SAT_TO_SATPOINT)
                  .unwrap()
                  .get(&sat)
                  .unwrap()
                  .unwrap()
                  .value()
              ),
              satpoint,
            );
          }
        }
      }
      None => {
        if self.has_sat_index().unwrap() {
          assert!(satpoint.outpoint == unbound_outpoint())
        }
      }
    }
  }

  fn inscriptions_on_output_unordered<'a: 'tx, 'tx>(
    satpoint_to_id: &'a impl ReadableMultimapTable<&'static SatPointValue, &'static InscriptionIdValue>,
    outpoint: OutPoint,
  ) -> Result<impl Iterator<Item = (SatPoint, InscriptionId)> + 'tx> {
    let start = SatPoint {
      outpoint,
      offset: 0,
    }
    .store();

    let end = SatPoint {
      outpoint,
      offset: u64::MAX,
    }
    .store();

    let mut inscriptions = Vec::new();

    for range in satpoint_to_id.range::<&[u8; 44]>(&start..=&end)? {
      let (satpoint, ids) = range?;
      for id_result in ids {
        let id = id_result?;
        inscriptions.push((Entry::load(*satpoint.value()), Entry::load(*id.value())));
      }
    }

    Ok(inscriptions.into_iter())
  }

  fn inscriptions_on_output_ordered<'a: 'tx, 'tx>(
    re_id_to_seq_num: &'a impl ReadableTable<&'static InscriptionIdValue, u64>,
    satpoint_to_id: &'a impl ReadableMultimapTable<&'static SatPointValue, &'static InscriptionIdValue>,
    outpoint: OutPoint,
  ) -> Result<Vec<(SatPoint, InscriptionId)>> {
    let mut result = Self::inscriptions_on_output_unordered(satpoint_to_id, outpoint)?
      .collect::<Vec<(SatPoint, InscriptionId)>>();

    if result.len() <= 1 {
      return Ok(result);
    }

    result.sort_by_key(|(_satpoint, inscription_id)| {
      match re_id_to_seq_num.get(&inscription_id.store()) {
        Ok(Some(num)) => num.value(),
        Ok(None) => 0,
        _ => 0,
      }
    });

    Ok(result)
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    bitcoin::secp256k1::rand::{self, RngCore},
  };

  struct ContextBuilder {
    args: Vec<OsString>,
    tempdir: Option<TempDir>,
  }

  impl ContextBuilder {
    fn build(self) -> Context {
      self.try_build().unwrap()
    }

    fn try_build(self) -> Result<Context> {
      let rpc_server = test_bitcoincore_rpc::builder()
        .network(Network::Regtest)
        .build();

      let tempdir = self.tempdir.unwrap_or_else(|| TempDir::new().unwrap());
      let cookie_file = tempdir.path().join("cookie");
      fs::write(&cookie_file, "username:password").unwrap();

      let command: Vec<OsString> = vec![
        "ord".into(),
        "--rpc-url".into(),
        rpc_server.url().into(),
        "--data-dir".into(),
        tempdir.path().into(),
        "--cookie-file".into(),
        cookie_file.into(),
        "--regtest".into(),
      ];

      let options = Options::try_parse_from(command.into_iter().chain(self.args)).unwrap();
      let index = Index::open(&options)?;
      index.update().unwrap();

      Ok(Context {
        options,
        rpc_server,
        tempdir,
        index,
      })
    }

    fn arg(mut self, arg: impl Into<OsString>) -> Self {
      self.args.push(arg.into());
      self
    }

    fn args<T: Into<OsString>, I: IntoIterator<Item = T>>(mut self, args: I) -> Self {
      self.args.extend(args.into_iter().map(|arg| arg.into()));
      self
    }

    fn tempdir(mut self, tempdir: TempDir) -> Self {
      self.tempdir = Some(tempdir);
      self
    }
  }

  struct Context {
    options: Options,
    rpc_server: test_bitcoincore_rpc::Handle,
    #[allow(unused)]
    tempdir: TempDir,
    index: Index,
  }

  impl Context {
    fn builder() -> ContextBuilder {
      ContextBuilder {
        args: Vec::new(),
        tempdir: None,
      }
    }

    fn mine_blocks(&self, n: u64) -> Vec<Block> {
      let blocks = self.rpc_server.mine_blocks(n);
      self.index.update().unwrap();
      blocks
    }

    fn mine_blocks_with_subsidy(&self, n: u64, subsidy: u64) -> Vec<Block> {
      let blocks = self.rpc_server.mine_blocks_with_subsidy(n, subsidy);
      self.index.update().unwrap();
      blocks
    }

    fn configurations() -> Vec<Context> {
      vec![
        Context::builder().build(),
        Context::builder().arg("--index-sats").build(),
      ]
    }
  }

  #[test]
  fn height_limit() {
    {
      let context = Context::builder().args(["--height-limit", "0"]).build();
      context.mine_blocks(1);
      assert_eq!(context.index.block_height().unwrap(), None);
      assert_eq!(context.index.block_count().unwrap(), 0);
    }

    {
      let context = Context::builder().args(["--height-limit", "1"]).build();
      context.mine_blocks(1);
      assert_eq!(context.index.block_height().unwrap(), Some(Height(0)));
      assert_eq!(context.index.block_count().unwrap(), 1);
    }

    {
      let context = Context::builder().args(["--height-limit", "2"]).build();
      context.mine_blocks(2);
      assert_eq!(context.index.block_height().unwrap(), Some(Height(1)));
      assert_eq!(context.index.block_count().unwrap(), 2);
    }
  }

  #[test]
  fn inscriptions_below_first_inscription_height_are_skipped() {
    let inscription = inscription("text/plain;charset=utf-8", "hello");
    let template = TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription.to_witness(),
      ..Default::default()
    };

    {
      let context = Context::builder().build();
      context.mine_blocks(1);
      let txid = context.rpc_server.broadcast_tx(template.clone());
      let inscription_id = InscriptionId::from(txid);
      context.mine_blocks(1);

      assert_eq!(
        context.index.get_inscription_by_id(inscription_id).unwrap(),
        Some(inscription)
      );

      assert_eq!(
        context
          .index
          .get_inscription_satpoint_by_id(inscription_id)
          .unwrap(),
        Some(SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        })
      );
    }

    {
      let context = Context::builder()
        .arg("--first-inscription-height=3")
        .build();
      context.mine_blocks(1);
      let txid = context.rpc_server.broadcast_tx(template);
      let inscription_id = InscriptionId::from(txid);
      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscription_satpoint_by_id(inscription_id)
          .unwrap(),
        None,
      );
    }
  }

  #[test]
  fn list_first_coinbase_transaction() {
    let context = Context::builder().arg("--index-sats").build();
    assert_eq!(
      context
        .index
        .list(
          "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0"
            .parse()
            .unwrap()
        )
        .unwrap()
        .unwrap(),
      List::Unspent(vec![(0, 50 * COIN_VALUE)])
    )
  }

  #[test]
  fn list_second_coinbase_transaction() {
    let context = Context::builder().arg("--index-sats").build();
    let txid = context.mine_blocks(1)[0].txdata[0].txid();
    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![(50 * COIN_VALUE, 100 * COIN_VALUE)])
    )
  }

  #[test]
  fn list_split_ranges_are_tracked_correctly() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(1);
    let split_coinbase_output = TransactionTemplate {
      inputs: &[(1, 0, 0)],
      outputs: 2,
      fee: 0,
      ..Default::default()
    };
    let txid = context.rpc_server.broadcast_tx(split_coinbase_output);

    context.mine_blocks(1);

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![(50 * COIN_VALUE, 75 * COIN_VALUE)])
    );

    assert_eq!(
      context.index.list(OutPoint::new(txid, 1)).unwrap().unwrap(),
      List::Unspent(vec![(75 * COIN_VALUE, 100 * COIN_VALUE)])
    );
  }

  #[test]
  fn list_merge_ranges_are_tracked_correctly() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(2);
    let merge_coinbase_outputs = TransactionTemplate {
      inputs: &[(1, 0, 0), (2, 0, 0)],
      fee: 0,
      ..Default::default()
    };

    let txid = context.rpc_server.broadcast_tx(merge_coinbase_outputs);
    context.mine_blocks(1);

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![
        (50 * COIN_VALUE, 100 * COIN_VALUE),
        (100 * COIN_VALUE, 150 * COIN_VALUE)
      ]),
    );
  }

  #[test]
  fn list_fee_paying_transaction_range() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(1);
    let fee_paying_tx = TransactionTemplate {
      inputs: &[(1, 0, 0)],
      outputs: 2,
      fee: 10,
      ..Default::default()
    };
    let txid = context.rpc_server.broadcast_tx(fee_paying_tx);
    let coinbase_txid = context.mine_blocks(1)[0].txdata[0].txid();

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![(50 * COIN_VALUE, 7499999995)]),
    );

    assert_eq!(
      context.index.list(OutPoint::new(txid, 1)).unwrap().unwrap(),
      List::Unspent(vec![(7499999995, 9999999990)]),
    );

    assert_eq!(
      context
        .index
        .list(OutPoint::new(coinbase_txid, 0))
        .unwrap()
        .unwrap(),
      List::Unspent(vec![(10000000000, 15000000000), (9999999990, 10000000000)])
    );
  }

  #[test]
  fn list_two_fee_paying_transaction_range() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(2);
    let first_fee_paying_tx = TransactionTemplate {
      inputs: &[(1, 0, 0)],
      fee: 10,
      ..Default::default()
    };
    let second_fee_paying_tx = TransactionTemplate {
      inputs: &[(2, 0, 0)],
      fee: 10,
      ..Default::default()
    };
    context.rpc_server.broadcast_tx(first_fee_paying_tx);
    context.rpc_server.broadcast_tx(second_fee_paying_tx);

    let coinbase_txid = context.mine_blocks(1)[0].txdata[0].txid();

    assert_eq!(
      context
        .index
        .list(OutPoint::new(coinbase_txid, 0))
        .unwrap()
        .unwrap(),
      List::Unspent(vec![
        (15000000000, 20000000000),
        (9999999990, 10000000000),
        (14999999990, 15000000000)
      ])
    );
  }

  #[test]
  fn list_null_output() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(1);
    let no_value_output = TransactionTemplate {
      inputs: &[(1, 0, 0)],
      fee: 50 * COIN_VALUE,
      ..Default::default()
    };
    let txid = context.rpc_server.broadcast_tx(no_value_output);
    context.mine_blocks(1);

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(Vec::new())
    );
  }

  #[test]
  fn list_null_input() {
    let context = Context::builder().arg("--index-sats").build();

    context.mine_blocks(1);
    let no_value_output = TransactionTemplate {
      inputs: &[(1, 0, 0)],
      fee: 50 * COIN_VALUE,
      ..Default::default()
    };
    context.rpc_server.broadcast_tx(no_value_output);
    context.mine_blocks(1);

    let no_value_input = TransactionTemplate {
      inputs: &[(2, 1, 0)],
      fee: 0,
      ..Default::default()
    };
    let txid = context.rpc_server.broadcast_tx(no_value_input);
    context.mine_blocks(1);

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(Vec::new())
    );
  }

  #[test]
  fn list_spent_output() {
    let context = Context::builder().arg("--index-sats").build();
    context.mine_blocks(1);
    context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      fee: 0,
      ..Default::default()
    });
    context.mine_blocks(1);
    let txid = context.rpc_server.tx(1, 0).txid();
    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Spent,
    );
  }

  #[test]
  fn list_unknown_output() {
    let context = Context::builder().arg("--index-sats").build();

    assert_eq!(
      context
        .index
        .list(
          "0000000000000000000000000000000000000000000000000000000000000000:0"
            .parse()
            .unwrap()
        )
        .unwrap(),
      None
    );
  }

  #[test]
  fn find_first_sat() {
    let context = Context::builder().arg("--index-sats").build();
    assert_eq!(
      context.index.find(0).unwrap().unwrap(),
      SatPoint {
        outpoint: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0"
          .parse()
          .unwrap(),
        offset: 0,
      }
    )
  }

  #[test]
  fn find_second_sat() {
    let context = Context::builder().arg("--index-sats").build();
    assert_eq!(
      context.index.find(1).unwrap().unwrap(),
      SatPoint {
        outpoint: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0"
          .parse()
          .unwrap(),
        offset: 1,
      }
    )
  }

  #[test]
  fn find_first_sat_of_second_block() {
    let context = Context::builder().arg("--index-sats").build();
    context.mine_blocks(1);
    assert_eq!(
      context.index.find(50 * COIN_VALUE).unwrap().unwrap(),
      SatPoint {
        outpoint: "30f2f037629c6a21c1f40ed39b9bd6278df39762d68d07f49582b23bcb23386a:0"
          .parse()
          .unwrap(),
        offset: 0,
      }
    )
  }

  #[test]
  fn find_unmined_sat() {
    let context = Context::builder().arg("--index-sats").build();
    assert_eq!(context.index.find(50 * COIN_VALUE).unwrap(), None);
  }

  #[test]
  fn find_first_sat_spent_in_second_block() {
    let context = Context::builder().arg("--index-sats").build();
    context.mine_blocks(1);
    let spend_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      fee: 0,
      ..Default::default()
    });
    context.mine_blocks(1);
    assert_eq!(
      context.index.find(50 * COIN_VALUE).unwrap().unwrap(),
      SatPoint {
        outpoint: OutPoint::new(spend_txid, 0),
        offset: 0,
      }
    )
  }

  #[test]
  fn inscriptions_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscriptions_without_sats_are_unbound() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        fee: 50 * 100_000_000,
        ..Default::default()
      });

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });

      let inscription_id = InscriptionId::from(txid);

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 0,
        },
        None,
      );

      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(4, 0, 0)],
        fee: 50 * 100_000_000,
        ..Default::default()
      });

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(5, 1, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });

      let inscription_id = InscriptionId::from(txid);

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 1,
        },
        None,
      );
    }
  }

  #[test]
  fn unaligned_inscriptions_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      let send_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0), (2, 1, 0)],
        ..Default::default()
      });

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: send_txid,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn merged_inscriptions_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(2);

      let first_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });

      let first_inscription_id = InscriptionId::from(first_txid);

      let second_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0)],
        witness: inscription("text/png", [1; 100]).to_witness(),
        ..Default::default()
      });
      let second_inscription_id = InscriptionId::from(second_txid);

      context.mine_blocks(1);

      let merged_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 0), (3, 2, 0)],
        ..Default::default()
      });

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        first_inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: merged_txid,
            vout: 0,
          },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        second_inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: merged_txid,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(100 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscriptions_that_are_sent_to_second_output_are_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      let send_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0), (2, 1, 0)],
        outputs: 2,
        ..Default::default()
      });

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: send_txid,
            vout: 1,
          },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn missing_inputs_are_fetched_from_bitcoin_core() {
    for args in [
      ["--first-inscription-height", "2"].as_slice(),
      ["--first-inscription-height", "2", "--index-sats"].as_slice(),
    ] {
      let context = Context::builder().args(args).build();
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      let send_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0), (2, 1, 0)],
        ..Default::default()
      });

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: send_txid,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn one_input_fee_spent_inscriptions_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);

      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0)],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });

      let coinbase_tx = context.mine_blocks(1)[0].txdata[0].txid();

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: coinbase_tx,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn two_input_fee_spent_inscriptions_are_tracked_correctly() {
    for context in Context::configurations() {
      context.mine_blocks(2);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);

      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0), (3, 1, 0)],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });

      let coinbase_tx = context.mine_blocks(1)[0].txdata[0].txid();

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: coinbase_tx,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscription_can_be_fee_spent_in_first_transaction() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        fee: 50 * COIN_VALUE,
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);

      let coinbase_tx = context.mine_blocks(1)[0].txdata[0].txid();

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: coinbase_tx,
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn lost_inscriptions() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        fee: 50 * COIN_VALUE,
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);

      context.mine_blocks_with_subsidy(1, 0);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint::null(),
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn multiple_inscriptions_can_be_lost() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let first_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        fee: 50 * COIN_VALUE,
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let first_inscription_id = InscriptionId::from(first_txid);

      context.mine_blocks_with_subsidy(1, 0);
      context.mine_blocks(1);

      let second_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 0, 0)],
        fee: 50 * COIN_VALUE,
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let second_inscription_id = InscriptionId::from(second_txid);

      context.mine_blocks_with_subsidy(1, 0);

      context.index.assert_inscription_location(
        first_inscription_id,
        SatPoint {
          outpoint: OutPoint::null(),
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        second_inscription_id,
        SatPoint {
          outpoint: OutPoint::null(),
          offset: 50 * COIN_VALUE,
        },
        Some(150 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn lost_sats_are_tracked_correctly() {
    let context = Context::builder().arg("--index-sats").build();
    assert_eq!(context.index.statistic(Statistic::LostSats), 0);

    context.mine_blocks(1);
    assert_eq!(context.index.statistic(Statistic::LostSats), 0);

    context.mine_blocks_with_subsidy(1, 0);
    assert_eq!(
      context.index.statistic(Statistic::LostSats),
      50 * COIN_VALUE
    );

    context.mine_blocks_with_subsidy(1, 0);
    assert_eq!(
      context.index.statistic(Statistic::LostSats),
      100 * COIN_VALUE
    );

    context.mine_blocks(1);
    assert_eq!(
      context.index.statistic(Statistic::LostSats),
      100 * COIN_VALUE
    );
  }

  #[test]
  fn lost_sat_ranges_are_tracked_correctly() {
    let context = Context::builder().arg("--index-sats").build();

    let null_ranges = || match context.index.list(OutPoint::null()).unwrap().unwrap() {
      List::Unspent(ranges) => ranges,
      _ => panic!(),
    };

    assert!(null_ranges().is_empty());

    context.mine_blocks(1);

    assert!(null_ranges().is_empty());

    context.mine_blocks_with_subsidy(1, 0);

    assert_eq!(null_ranges(), [(100 * COIN_VALUE, 150 * COIN_VALUE)]);

    context.mine_blocks_with_subsidy(1, 0);

    assert_eq!(
      null_ranges(),
      [
        (100 * COIN_VALUE, 150 * COIN_VALUE),
        (150 * COIN_VALUE, 200 * COIN_VALUE)
      ]
    );

    context.mine_blocks(1);

    assert_eq!(
      null_ranges(),
      [
        (100 * COIN_VALUE, 150 * COIN_VALUE),
        (150 * COIN_VALUE, 200 * COIN_VALUE)
      ]
    );

    context.mine_blocks_with_subsidy(1, 0);

    assert_eq!(
      null_ranges(),
      [
        (100 * COIN_VALUE, 150 * COIN_VALUE),
        (150 * COIN_VALUE, 200 * COIN_VALUE),
        (250 * COIN_VALUE, 300 * COIN_VALUE)
      ]
    );
  }

  #[test]
  fn lost_inscriptions_get_lost_satpoints() {
    for context in Context::configurations() {
      context.mine_blocks_with_subsidy(1, 0);
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0)],
        outputs: 2,
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);
      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 1), (3, 1, 0)],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      context.mine_blocks_with_subsidy(1, 0);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint::null(),
          offset: 75 * COIN_VALUE,
        },
        Some(100 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscription_skips_zero_value_first_output_of_inscribe_transaction() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        outputs: 2,
        witness: inscription("text/plain", "hello").to_witness(),
        output_values: &[0, 50 * COIN_VALUE],
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);
      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 1 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscription_can_be_lost_in_first_transaction() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        fee: 50 * COIN_VALUE,
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);
      context.mine_blocks_with_subsidy(1, 0);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint::null(),
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn lost_rare_sats_are_tracked() {
    let context = Context::builder().arg("--index-sats").build();
    context.mine_blocks_with_subsidy(1, 0);
    context.mine_blocks_with_subsidy(1, 0);

    assert_eq!(
      context
        .index
        .rare_sat_satpoint(Sat(50 * COIN_VALUE))
        .unwrap()
        .unwrap(),
      SatPoint {
        outpoint: OutPoint::null(),
        offset: 0,
      },
    );

    assert_eq!(
      context
        .index
        .rare_sat_satpoint(Sat(100 * COIN_VALUE))
        .unwrap()
        .unwrap(),
      SatPoint {
        outpoint: OutPoint::null(),
        offset: 50 * COIN_VALUE,
      },
    );
  }

  #[test]
  fn old_schema_gives_correct_error() {
    let tempdir = {
      let context = Context::builder().build();

      let wtx = context.index.database.begin_write().unwrap();

      wtx
        .open_table(STATISTIC_TO_COUNT)
        .unwrap()
        .insert(&Statistic::Schema.key(), &0)
        .unwrap();

      wtx.commit().unwrap();

      context.tempdir
    };

    let path = tempdir.path().to_owned();

    let delimiter = if cfg!(windows) { '\\' } else { '/' };

    assert_eq!(
      Context::builder().tempdir(tempdir).try_build().err().unwrap().to_string(),
      format!("index at `{}{delimiter}regtest{delimiter}index.redb` appears to have been built with an older, incompatible version of ord, consider deleting and rebuilding the index: index schema 0, ord schema {SCHEMA_VERSION}", path.display()));
  }

  #[test]
  fn new_schema_gives_correct_error() {
    let tempdir = {
      let context = Context::builder().build();

      let wtx = context.index.database.begin_write().unwrap();

      wtx
        .open_table(STATISTIC_TO_COUNT)
        .unwrap()
        .insert(&Statistic::Schema.key(), &u64::MAX)
        .unwrap();

      wtx.commit().unwrap();

      context.tempdir
    };

    let path = tempdir.path().to_owned();

    let delimiter = if cfg!(windows) { '\\' } else { '/' };

    assert_eq!(
      Context::builder().tempdir(tempdir).try_build().err().unwrap().to_string(),
      format!("index at `{}{delimiter}regtest{delimiter}index.redb` appears to have been built with a newer, incompatible version of ord, consider updating ord: index schema {}, ord schema {SCHEMA_VERSION}", path.display(), u64::MAX));
  }

  #[test]
  fn inscriptions_on_output() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });

      let inscription_id = InscriptionId::from(txid);

      assert_eq!(
        context
          .index
          .get_inscriptions_on_output(OutPoint { txid, vout: 0 })
          .unwrap(),
        []
      );

      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscriptions_on_output(OutPoint { txid, vout: 0 })
          .unwrap(),
        [inscription_id]
      );

      let send_id = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0)],
        ..Default::default()
      });

      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscriptions_on_output(OutPoint { txid, vout: 0 })
          .unwrap(),
        []
      );

      assert_eq!(
        context
          .index
          .get_inscriptions_on_output(OutPoint {
            txid: send_id,
            vout: 0,
          })
          .unwrap(),
        [inscription_id]
      );
    }
  }

  #[test]
  fn inscriptions_on_same_sat_after_the_first_are_not_unbound() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let first = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId::from(first);

      assert_eq!(
        context
          .index
          .get_inscriptions_on_output(OutPoint {
            txid: first,
            vout: 0
          })
          .unwrap(),
        [inscription_id]
      );

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: first,
            vout: 0,
          },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      let second = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });

      let inscription_id = InscriptionId::from(second);

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: second,
            vout: 0,
          },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      assert!(context
        .index
        .get_inscription_entry(second.into())
        .unwrap()
        .is_some());

      assert!(context
        .index
        .get_inscription_by_id(second.into())
        .unwrap()
        .is_some());
    }
  }

  #[test]
  fn get_latest_inscriptions_with_no_prev_and_next() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);

      context.mine_blocks(1);

      let (inscriptions, prev, next) = context
        .index
        .get_latest_inscriptions_with_prev_and_next(100, None)
        .unwrap();
      assert_eq!(inscriptions, &[inscription_id]);
      assert_eq!(prev, None);
      assert_eq!(next, None);
    }
  }

  #[test]
  fn get_latest_inscriptions_with_prev_and_next() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let mut ids = Vec::new();

      for i in 0..103 {
        let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
          inputs: &[(i + 1, 0, 0)],
          witness: inscription("text/plain", "hello").to_witness(),
          ..Default::default()
        });
        ids.push(InscriptionId::from(txid));
        context.mine_blocks(1);
      }

      ids.reverse();

      let (inscriptions, prev, next) = context
        .index
        .get_latest_inscriptions_with_prev_and_next(100, None)
        .unwrap();
      assert_eq!(inscriptions, &ids[..100]);
      assert_eq!(prev, Some(2));
      assert_eq!(next, None);

      let (inscriptions, prev, next) = context
        .index
        .get_latest_inscriptions_with_prev_and_next(100, Some(101))
        .unwrap();
      assert_eq!(inscriptions, &ids[1..101]);
      assert_eq!(prev, Some(1));
      assert_eq!(next, Some(102));

      let (inscriptions, prev, next) = context
        .index
        .get_latest_inscriptions_with_prev_and_next(100, Some(0))
        .unwrap();
      assert_eq!(inscriptions, &ids[102..103]);
      assert_eq!(prev, None);
      assert_eq!(next, Some(100));
    }
  }

  #[test]
  fn unsynced_index_fails() {
    for context in Context::configurations() {
      let mut entropy = [0; 16];
      rand::thread_rng().fill_bytes(&mut entropy);
      let mnemonic = Mnemonic::from_entropy(&entropy).unwrap();
      crate::subcommand::wallet::initialize_wallet(&context.options, mnemonic.to_seed("")).unwrap();
      context.rpc_server.mine_blocks(1);
      assert_regex_match!(
        context
          .index
          .get_unspent_outputs(Wallet::load(&context.options).unwrap())
          .unwrap_err()
          .to_string(),
        r"output in Bitcoin Core wallet but not in ord index: [[:xdigit:]]{64}:\d+"
      );
    }
  }

  #[test]
  // https://github.com/ordinals/ord/issues/2062
  fn zero_value_transaction_inscription_not_cursed_but_unbound() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        fee: 50 * 100_000_000,
        ..Default::default()
      });

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0)],
        witness: inscription("text/plain", "hello").to_witness(),
        ..Default::default()
      });

      let inscription_id = InscriptionId::from(txid);

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 0,
        },
        None,
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .number,
        0
      );
    }
  }

  #[test]
  fn multiple_inscriptions_in_same_tx_all_but_first_input_are_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);
      context.mine_blocks(1);
      context.mine_blocks(1);

      let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0), (2, 0, 0), (3, 0, 0)],
        witness,
        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 };
      let second = InscriptionId { txid, index: 1 };
      let third = InscriptionId { txid, index: 2 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        first,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        second,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 50 * COIN_VALUE,
        },
        Some(100 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        third,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 100 * COIN_VALUE,
        },
        Some(150 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(first)
          .unwrap()
          .unwrap()
          .number,
        0
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second)
          .unwrap()
          .unwrap()
          .number,
        -1
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(third)
          .unwrap()
          .unwrap()
          .number,
        -2
      );
    }
  }

  #[test]
  fn multiple_inscriptions_same_input_all_but_first_are_cursed_and_unbound() {
    for context in Context::configurations() {
      context.rpc_server.mine_blocks(1);

      let script = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice(&[1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice(&[])
        .push_slice(b"foo")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice(&[1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice(&[])
        .push_slice(b"bar")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice(&[1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice(&[])
        .push_slice(b"qix")
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_vec(vec![script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness,
        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 };
      let second = InscriptionId { txid, index: 1 };
      let third = InscriptionId { txid, index: 2 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        first,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        second,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 0,
        },
        None,
      );

      context.index.assert_inscription_location(
        third,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 1,
        },
        None,
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(first)
          .unwrap()
          .unwrap()
          .number,
        0
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second)
          .unwrap()
          .unwrap()
          .number,
        -1
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(third)
          .unwrap()
          .unwrap()
          .number,
        -2
      );
    }
  }

  #[test]
  fn multiple_inscriptions_different_inputs_and_same_inputs() {
    for context in Context::configurations() {
      context.rpc_server.mine_blocks(1);
      context.rpc_server.mine_blocks(1);
      context.rpc_server.mine_blocks(1);

      let script = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice(&[1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice(&[])
        .push_slice(b"foo")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice(&[1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice(&[])
        .push_slice(b"bar")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice(&[1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice(&[])
        .push_slice(b"qix")
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_vec(vec![script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0), (2, 0, 0), (3, 0, 0)],
        witness, // the witness is replicated over all inputs
        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 }; // normal
      let fourth = InscriptionId { txid, index: 3 }; // cursed but bound
      let ninth = InscriptionId { txid, index: 8 }; // cursed and unbound

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        first,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        fourth,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 50 * COIN_VALUE,
        },
        Some(100 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        ninth,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 5,
        },
        None,
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(first)
          .unwrap()
          .unwrap()
          .number,
        0
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(fourth)
          .unwrap()
          .unwrap()
          .number,
        -3
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(ninth)
          .unwrap()
          .unwrap()
          .number,
        -8
      );
    }
  }

  #[test]
  fn genesis_fee_distributed_evenly() {
    for context in Context::configurations() {
      context.rpc_server.mine_blocks(1);

      let script = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice(&[1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice(&[])
        .push_slice(b"foo")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice(&[1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice(&[])
        .push_slice(b"bar")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice(&[1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice(&[])
        .push_slice(b"qix")
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_vec(vec![script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness,
        fee: 33,
        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 };
      let second = InscriptionId { txid, index: 1 };

      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscription_entry(first)
          .unwrap()
          .unwrap()
          .fee,
        11
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second)
          .unwrap()
          .unwrap()
          .fee,
        11
      );
    }
  }

  #[test]
  fn reinscription_on_cursed_inscription_is_not_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);
      context.mine_blocks(1);

      let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

      let cursed_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0), (2, 0, 0)],
        witness,
        outputs: 2,
        ..Default::default()
      });

      let cursed = InscriptionId {
        txid: cursed_txid,
        index: 1,
      };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        cursed,
        SatPoint {
          outpoint: OutPoint {
            txid: cursed_txid,
            vout: 1,
          },
          offset: 0,
        },
        Some(100 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(cursed)
          .unwrap()
          .unwrap()
          .number,
        -1
      );

      let witness = envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"reinscription on cursed",
      ]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 1)],
        witness,
        ..Default::default()
      });

      let reinscription_on_cursed = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        reinscription_on_cursed,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(100 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(reinscription_on_cursed)
          .unwrap()
          .unwrap()
          .number,
        1
      );
    }
  }

  #[test]
  fn second_reinscription_on_cursed_inscription_is_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);
      context.mine_blocks(1);

      let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

      let cursed_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0), (2, 0, 0)],
        witness,
        outputs: 2,
        ..Default::default()
      });

      let cursed = InscriptionId {
        txid: cursed_txid,
        index: 1,
      };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        cursed,
        SatPoint {
          outpoint: OutPoint {
            txid: cursed_txid,
            vout: 1,
          },
          offset: 0,
        },
        Some(100 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(cursed)
          .unwrap()
          .unwrap()
          .number,
        -1
      );

      let witness = envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"reinscription on cursed",
      ]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 1)],
        witness,
        ..Default::default()
      });

      let reinscription_on_cursed = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        reinscription_on_cursed,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(100 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(reinscription_on_cursed)
          .unwrap()
          .unwrap()
          .number,
        1
      );

      let witness = envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"second reinscription on cursed",
      ]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(4, 1, 0)],
        witness,
        ..Default::default()
      });

      let second_reinscription_on_cursed = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        second_reinscription_on_cursed,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(100 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second_reinscription_on_cursed)
          .unwrap()
          .unwrap()
          .number,
        -2
      );

      assert_eq!(
        vec![
          cursed,
          reinscription_on_cursed,
          second_reinscription_on_cursed
        ],
        context
          .index
          .get_inscriptions_on_output_with_satpoints(OutPoint { txid, vout: 0 })
          .unwrap()
          .iter()
          .map(|(_satpoint, inscription_id)| *inscription_id)
          .collect::<Vec<InscriptionId>>()
      )
    }
  }

  #[test]
  fn reinscriptions_on_output_correctly_ordered_and_transferred() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription("text/plain;charset=utf-8", "hello").to_witness(),
        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0)],
        witness: inscription("text/plain;charset=utf-8", "hello").to_witness(),
        ..Default::default()
      });

      let second = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);
      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 0)],
        witness: inscription("text/plain;charset=utf-8", "hello").to_witness(),
        ..Default::default()
      });

      let third = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      let location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      assert_eq!(
        vec![(location, first), (location, second), (location, third)],
        context
          .index
          .get_inscriptions_on_output_with_satpoints(OutPoint { txid, vout: 0 })
          .unwrap()
      )
    }
  }
}
