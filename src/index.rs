use {
  self::{
    entry::{
      Entry, HeaderValue, InscriptionEntry, InscriptionEntryValue, InscriptionIdValue,
      OutPointValue, RuneEntryValue, RuneIdValue, SatPointValue, SatRange, TxidValue,
    },
    reorg::*,
    runes::{Rune, RuneId},
    updater::Updater,
  },
  super::*,
  crate::{
    subcommand::{find::FindRangeOutput, server::InscriptionQuery},
    templates::StatusHtml,
  },
  bitcoin::block::Header,
  bitcoincore_rpc::{json::GetBlockHeaderResult, Client},
  chrono::SubsecRound,
  indicatif::{ProgressBar, ProgressStyle},
  log::log_enabled,
  redb::{
    Database, DatabaseError, MultimapTable, MultimapTableDefinition, MultimapTableHandle,
    ReadOnlyTable, ReadableMultimapTable, ReadableTable, RedbKey, RedbValue, RepairSession,
    StorageError, Table, TableDefinition, TableHandle, WriteTransaction,
  },
  std::{
    collections::{BTreeSet, HashMap},
    io::{BufWriter, Write},
    sync::{Mutex, Once},
  },
};

pub use self::entry::RuneEntry;

pub(crate) mod entry;
mod fetcher;
mod reorg;
mod rtx;
mod updater;

#[cfg(test)]
pub(crate) mod testing;

const SCHEMA_VERSION: u64 = 16;

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

define_multimap_table! { SATPOINT_TO_SEQUENCE_NUMBER, &SatPointValue, u32 }
define_multimap_table! { SAT_TO_SEQUENCE_NUMBER, u64, u32 }
define_multimap_table! { SEQUENCE_NUMBER_TO_CHILDREN, u32, u32 }
define_table! { HEIGHT_TO_BLOCK_HEADER, u32, &HeaderValue }
define_table! { HEIGHT_TO_LAST_SEQUENCE_NUMBER, u32, u32 }
define_table! { HOME_INSCRIPTIONS, u32, InscriptionIdValue }
define_table! { INSCRIPTION_ID_TO_SEQUENCE_NUMBER, InscriptionIdValue, u32 }
define_table! { INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER, i32, u32 }
define_table! { OUTPOINT_TO_RUNE_BALANCES, &OutPointValue, &[u8] }
define_table! { OUTPOINT_TO_SAT_RANGES, &OutPointValue, &[u8] }
define_table! { OUTPOINT_TO_VALUE, &OutPointValue, u64}
define_table! { RUNE_ID_TO_RUNE_ENTRY, RuneIdValue, RuneEntryValue }
define_table! { RUNE_TO_RUNE_ID, u128, RuneIdValue }
define_table! { SAT_TO_SATPOINT, u64, &SatPointValue }
define_table! { SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY, u32, InscriptionEntryValue }
define_table! { SEQUENCE_NUMBER_TO_RUNE_ID, u32, RuneIdValue }
define_table! { SEQUENCE_NUMBER_TO_SATPOINT, u32, &SatPointValue }
define_table! { STATISTIC_TO_COUNT, u64, u64 }
define_table! { TRANSACTION_ID_TO_RUNE, &TxidValue, u128 }
define_table! { TRANSACTION_ID_TO_TRANSACTION, &TxidValue, &[u8] }
define_table! { WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP, u32, u128 }

#[derive(Debug, PartialEq)]
pub enum List {
  Spent,
  Unspent(Vec<(u64, u64)>),
}

#[derive(Copy, Clone)]
pub(crate) enum Statistic {
  Schema = 0,
  BlessedInscriptions = 1,
  Commits = 2,
  CursedInscriptions = 3,
  IndexRunes = 4,
  IndexSats = 5,
  LostSats = 6,
  OutputsTraversed = 7,
  ReservedRunes = 8,
  Runes = 9,
  SatRanges = 10,
  UnboundInscriptions = 11,
  IndexTransactions = 12,
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
  blocks_indexed: u32,
  branch_pages: u64,
  fragmented_bytes: u64,
  index_file_size: u64,
  index_path: PathBuf,
  leaf_pages: u64,
  metadata_bytes: u64,
  outputs_traversed: u64,
  page_size: usize,
  sat_ranges: u64,
  stored_bytes: u64,
  tables: BTreeMap<String, TableInfo>,
  total_bytes: u64,
  pub(crate) transactions: Vec<TransactionInfo>,
  tree_height: u32,
  utxos_indexed: u64,
}

#[derive(Serialize)]
pub(crate) struct TableInfo {
  branch_pages: u64,
  fragmented_bytes: u64,
  leaf_pages: u64,
  metadata_bytes: u64,
  proportion: f64,
  stored_bytes: u64,
  total_bytes: u64,
  tree_height: u32,
}

#[derive(Serialize)]
pub(crate) struct TransactionInfo {
  pub(crate) starting_block_count: u32,
  pub(crate) starting_timestamp: u128,
}

pub(crate) struct InscriptionInfo {
  pub(crate) children: Vec<InscriptionId>,
  pub(crate) entry: InscriptionEntry,
  pub(crate) parent: Option<InscriptionId>,
  pub(crate) output: Option<TxOut>,
  pub(crate) satpoint: SatPoint,
  pub(crate) inscription: Inscription,
  pub(crate) previous: Option<InscriptionId>,
  pub(crate) next: Option<InscriptionId>,
  pub(crate) rune: Option<SpacedRune>,
  pub(crate) charms: u16,
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

pub struct Index {
  client: Client,
  database: Database,
  durability: redb::Durability,
  first_inscription_height: u32,
  genesis_block_coinbase_transaction: Transaction,
  genesis_block_coinbase_txid: Txid,
  height_limit: Option<u32>,
  index_runes: bool,
  index_sats: bool,
  index_transactions: bool,
  options: Options,
  path: PathBuf,
  started: DateTime<Utc>,
  unrecoverably_reorged: AtomicBool,
}

impl Index {
  pub fn open(options: &Options) -> Result<Self> {
    let client = options.bitcoin_rpc_client(None)?;

    let path = options
      .index
      .clone()
      .unwrap_or(options.data_dir().clone().join("index.redb"));

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

    let durability = if cfg!(test) {
      redb::Durability::None
    } else {
      redb::Durability::Immediate
    };

    let index_runes;
    let index_sats;
    let index_transactions;

    let index_path = path.clone();
    let once = Once::new();
    let progress_bar = Mutex::new(None);

    let database = match Database::builder()
      .set_cache_size(db_cache_size)
      .set_repair_callback(move |progress: &mut RepairSession| {
        once.call_once(|| println!("Index file `{}` needs recovery. This can take a long time, especially for the --index-sats index.", index_path.display()));

        if !(cfg!(test) || log_enabled!(log::Level::Info) || integration_test()) {
          let mut guard = progress_bar.lock().unwrap();

          let progress_bar = guard.get_or_insert_with(|| {
            let progress_bar = ProgressBar::new(100);
            progress_bar.set_style(
              ProgressStyle::with_template("[repairing database] {wide_bar} {pos}/{len}").unwrap(),
            );
            progress_bar
          });

          #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
          progress_bar.set_position((progress.progress() * 100.0) as u64);
        }
      })
      .open(&path)
    {
      Ok(database) => {
        {
          let tx = database.begin_read()?;
          let statistics = tx.open_table(STATISTIC_TO_COUNT)?;

          let schema_version = statistics
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


          index_runes = Self::is_statistic_set(&statistics, Statistic::IndexRunes)?;
          index_sats = Self::is_statistic_set(&statistics, Statistic::IndexSats)?;
          index_transactions = Self::is_statistic_set(&statistics, Statistic::IndexTransactions)?;
        }

        database
      }
      Err(DatabaseError::Storage(StorageError::Io(error)))
        if error.kind() == io::ErrorKind::NotFound =>
      {
        let database = Database::builder()
          .set_cache_size(db_cache_size)
          .create(&path)?;

        let mut tx = database.begin_write()?;

        tx.set_durability(durability);

        tx.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER)?;
        tx.open_multimap_table(SAT_TO_SEQUENCE_NUMBER)?;
        tx.open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN)?;
        tx.open_table(HEIGHT_TO_BLOCK_HEADER)?;
        tx.open_table(HEIGHT_TO_LAST_SEQUENCE_NUMBER)?;
        tx.open_table(HOME_INSCRIPTIONS)?;
        tx.open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;
        tx.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER)?;
        tx.open_table(OUTPOINT_TO_RUNE_BALANCES)?;
        tx.open_table(OUTPOINT_TO_VALUE)?;
        tx.open_table(RUNE_ID_TO_RUNE_ENTRY)?;
        tx.open_table(RUNE_TO_RUNE_ID)?;
        tx.open_table(SAT_TO_SATPOINT)?;
        tx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;
        tx.open_table(SEQUENCE_NUMBER_TO_RUNE_ID)?;
        tx.open_table(SEQUENCE_NUMBER_TO_SATPOINT)?;
        tx.open_table(TRANSACTION_ID_TO_RUNE)?;
        tx.open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?;

        {
          let mut outpoint_to_sat_ranges = tx.open_table(OUTPOINT_TO_SAT_RANGES)?;
          let mut statistics = tx.open_table(STATISTIC_TO_COUNT)?;

          if options.index_sats {
            outpoint_to_sat_ranges.insert(&OutPoint::null().store(), [].as_slice())?;
          }

          index_runes = options.index_runes();
          index_sats = options.index_sats;
          index_transactions = options.index_transactions;

          Self::set_statistic(&mut statistics, Statistic::IndexRunes, u64::from(index_runes))?;
          Self::set_statistic(&mut statistics, Statistic::IndexSats, u64::from(index_sats))?;
          Self::set_statistic(&mut statistics, Statistic::IndexTransactions, u64::from(index_transactions))?;
          Self::set_statistic(&mut statistics, Statistic::Schema, SCHEMA_VERSION)?;
        }

        tx.commit()?;

        database
      }
      Err(error) => bail!("failed to open index: {error}"),
    };

    let genesis_block_coinbase_transaction =
      options.chain().genesis_block().coinbase().unwrap().clone();

    Ok(Self {
      genesis_block_coinbase_txid: genesis_block_coinbase_transaction.txid(),
      client,
      database,
      durability,
      first_inscription_height: options.first_inscription_height(),
      genesis_block_coinbase_transaction,
      height_limit: options.height_limit,
      index_runes,
      index_sats,
      index_transactions,
      options: options.clone(),
      path,
      started: Utc::now(),
      unrecoverably_reorged: AtomicBool::new(false),
    })
  }

  #[cfg(test)]
  fn set_durability(&mut self, durability: redb::Durability) {
    self.durability = durability;
  }

  pub(crate) fn check_sync(&self, utxos: &BTreeMap<OutPoint, Amount>) -> Result<bool> {
    let rtx = self.database.begin_read()?;
    let outpoint_to_value = rtx.open_table(OUTPOINT_TO_VALUE)?;
    for outpoint in utxos.keys() {
      if outpoint_to_value.get(&outpoint.store())?.is_none() {
        return Err(anyhow!(
          "output in Bitcoin Core wallet but not in ord index: {outpoint}"
        ));
      }
    }

    Ok(true)
  }

  pub(crate) fn has_rune_index(&self) -> bool {
    self.index_runes
  }

  pub(crate) fn has_sat_index(&self) -> bool {
    self.index_sats
  }

  pub(crate) fn status(&self) -> Result<StatusHtml> {
    let rtx = self.database.begin_read()?;

    let statistic_to_count = rtx.open_table(STATISTIC_TO_COUNT)?;

    let statistic = |statistic: Statistic| -> Result<u64> {
      Ok(
        statistic_to_count
          .get(statistic.key())?
          .map(|guard| guard.value())
          .unwrap_or_default(),
      )
    };

    let height = rtx
      .open_table(HEIGHT_TO_BLOCK_HEADER)?
      .range(0..)?
      .next_back()
      .transpose()?
      .map(|(height, _header)| height.value());

    let next_height = height.map(|height| height + 1).unwrap_or(0);

    let blessed_inscriptions = statistic(Statistic::BlessedInscriptions)?;
    let cursed_inscriptions = statistic(Statistic::CursedInscriptions)?;

    Ok(StatusHtml {
      blessed_inscriptions,
      chain: self.options.chain(),
      cursed_inscriptions,
      height,
      inscriptions: blessed_inscriptions + cursed_inscriptions,
      lost_sats: statistic(Statistic::LostSats)?,
      minimum_rune_for_next_block: Rune::minimum_at_height(
        self.options.chain(),
        Height(next_height),
      ),
      rune_index: statistic(Statistic::IndexRunes)? != 0,
      runes: statistic(Statistic::Runes)?,
      sat_index: statistic(Statistic::IndexSats)? != 0,
      started: self.started,
      transaction_index: statistic(Statistic::IndexTransactions)? != 0,
      unrecoverably_reorged: self.unrecoverably_reorged.load(atomic::Ordering::Relaxed),
      uptime: (Utc::now() - self.started).to_std()?,
    })
  }

  pub(crate) fn info(&self) -> Result<Info> {
    fn insert_table_info<K: RedbKey + 'static, V: RedbValue + 'static>(
      tables: &mut BTreeMap<String, TableInfo>,
      wtx: &WriteTransaction,
      database_total_bytes: u64,
      definition: TableDefinition<K, V>,
    ) {
      let stats = wtx.open_table(definition).unwrap().stats().unwrap();

      let fragmented_bytes = stats.fragmented_bytes();
      let metadata_bytes = stats.metadata_bytes();
      let stored_bytes = stats.stored_bytes();
      let total_bytes = stored_bytes + metadata_bytes + fragmented_bytes;

      tables.insert(
        definition.name().into(),
        TableInfo {
          branch_pages: stats.branch_pages(),
          fragmented_bytes,
          leaf_pages: stats.leaf_pages(),
          metadata_bytes,
          proportion: total_bytes as f64 / database_total_bytes as f64,
          stored_bytes,
          total_bytes,
          tree_height: stats.tree_height(),
        },
      );
    }

    fn insert_multimap_table_info<K: RedbKey + 'static, V: RedbValue + RedbKey + 'static>(
      tables: &mut BTreeMap<String, TableInfo>,
      wtx: &WriteTransaction,
      database_total_bytes: u64,
      definition: MultimapTableDefinition<K, V>,
    ) {
      let stats = wtx
        .open_multimap_table(definition)
        .unwrap()
        .stats()
        .unwrap();

      let fragmented_bytes = stats.fragmented_bytes();
      let metadata_bytes = stats.metadata_bytes();
      let stored_bytes = stats.stored_bytes();
      let total_bytes = stored_bytes + metadata_bytes + fragmented_bytes;

      tables.insert(
        definition.name().into(),
        TableInfo {
          branch_pages: stats.branch_pages(),
          fragmented_bytes,
          leaf_pages: stats.leaf_pages(),
          metadata_bytes,
          proportion: total_bytes as f64 / database_total_bytes as f64,
          stored_bytes,
          total_bytes,
          tree_height: stats.tree_height(),
        },
      );
    }

    let wtx = self.begin_write()?;

    let stats = wtx.stats()?;

    let fragmented_bytes = stats.fragmented_bytes();
    let metadata_bytes = stats.metadata_bytes();
    let stored_bytes = stats.stored_bytes();
    let total_bytes = fragmented_bytes + metadata_bytes + stored_bytes;

    let mut tables: BTreeMap<String, TableInfo> = BTreeMap::new();

    insert_multimap_table_info(&mut tables, &wtx, total_bytes, SATPOINT_TO_SEQUENCE_NUMBER);
    insert_multimap_table_info(&mut tables, &wtx, total_bytes, SAT_TO_SEQUENCE_NUMBER);
    insert_multimap_table_info(&mut tables, &wtx, total_bytes, SEQUENCE_NUMBER_TO_CHILDREN);
    insert_table_info(&mut tables, &wtx, total_bytes, HEIGHT_TO_BLOCK_HEADER);
    insert_table_info(
      &mut tables,
      &wtx,
      total_bytes,
      HEIGHT_TO_LAST_SEQUENCE_NUMBER,
    );
    insert_table_info(&mut tables, &wtx, total_bytes, HOME_INSCRIPTIONS);
    insert_table_info(
      &mut tables,
      &wtx,
      total_bytes,
      INSCRIPTION_ID_TO_SEQUENCE_NUMBER,
    );
    insert_table_info(
      &mut tables,
      &wtx,
      total_bytes,
      INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER,
    );
    insert_table_info(&mut tables, &wtx, total_bytes, OUTPOINT_TO_RUNE_BALANCES);
    insert_table_info(&mut tables, &wtx, total_bytes, OUTPOINT_TO_SAT_RANGES);
    insert_table_info(&mut tables, &wtx, total_bytes, OUTPOINT_TO_VALUE);
    insert_table_info(&mut tables, &wtx, total_bytes, RUNE_ID_TO_RUNE_ENTRY);
    insert_table_info(&mut tables, &wtx, total_bytes, RUNE_TO_RUNE_ID);
    insert_table_info(&mut tables, &wtx, total_bytes, SAT_TO_SATPOINT);
    insert_table_info(
      &mut tables,
      &wtx,
      total_bytes,
      SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY,
    );
    insert_table_info(&mut tables, &wtx, total_bytes, SEQUENCE_NUMBER_TO_RUNE_ID);
    insert_table_info(&mut tables, &wtx, total_bytes, SEQUENCE_NUMBER_TO_SATPOINT);
    insert_table_info(&mut tables, &wtx, total_bytes, STATISTIC_TO_COUNT);
    insert_table_info(&mut tables, &wtx, total_bytes, TRANSACTION_ID_TO_RUNE);
    insert_table_info(
      &mut tables,
      &wtx,
      total_bytes,
      TRANSACTION_ID_TO_TRANSACTION,
    );
    insert_table_info(
      &mut tables,
      &wtx,
      total_bytes,
      WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP,
    );

    for table in wtx.list_tables()? {
      assert!(tables.contains_key(table.name()));
    }

    for table in wtx.list_multimap_tables()? {
      assert!(tables.contains_key(table.name()));
    }

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
        blocks_indexed: wtx
          .open_table(HEIGHT_TO_BLOCK_HEADER)?
          .range(0..)?
          .next_back()
          .and_then(|result| result.ok())
          .map(|(height, _header)| height.value() + 1)
          .unwrap_or(0),
        branch_pages: stats.branch_pages(),
        fragmented_bytes,
        index_file_size: fs::metadata(&self.path)?.len(),
        index_path: self.path.clone(),
        leaf_pages: stats.leaf_pages(),
        metadata_bytes,
        outputs_traversed,
        page_size: stats.page_size(),
        sat_ranges,
        stored_bytes,
        tables,
        total_bytes,
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
    let mut updater = Updater::new(self)?;

    loop {
      match updater.update_index() {
        Ok(ok) => return Ok(ok),
        Err(err) => {
          log::info!("{}", err.to_string());

          match err.downcast_ref() {
            Some(&ReorgError::Recoverable { height, depth }) => {
              Reorg::handle_reorg(self, height, depth)?;

              updater = Updater::new(self)?;
            }
            Some(&ReorgError::Unrecoverable) => {
              self
                .unrecoverably_reorged
                .store(true, atomic::Ordering::Relaxed);
              return Err(anyhow!(ReorgError::Unrecoverable));
            }
            _ => return Err(err),
          };
        }
      }
    }
  }

  pub(crate) fn export(&self, filename: &String, include_addresses: bool) -> Result {
    let mut writer = BufWriter::new(File::create(filename)?);
    let rtx = self.database.begin_read()?;

    let blocks_indexed = rtx
      .open_table(HEIGHT_TO_BLOCK_HEADER)?
      .range(0..)?
      .next_back()
      .and_then(|result| result.ok())
      .map(|(height, _header)| height.value() + 1)
      .unwrap_or(0);

    writeln!(writer, "# export at block height {}", blocks_indexed)?;

    log::info!("exporting database tables to {filename}");

    let sequence_number_to_satpoint = rtx.open_table(SEQUENCE_NUMBER_TO_SATPOINT)?;

    for result in rtx
      .open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?
      .iter()?
    {
      let entry = result?;
      let sequence_number = entry.0.value();
      let entry = InscriptionEntry::load(entry.1.value());
      let satpoint = SatPoint::load(
        *sequence_number_to_satpoint
          .get(sequence_number)?
          .unwrap()
          .value(),
      );

      write!(
        writer,
        "{}\t{}\t{}",
        entry.inscription_number, entry.id, satpoint
      )?;

      if include_addresses {
        let address = if satpoint.outpoint == unbound_outpoint() {
          "unbound".to_string()
        } else {
          let output = self
            .get_transaction(satpoint.outpoint.txid)?
            .unwrap()
            .output
            .into_iter()
            .nth(satpoint.outpoint.vout.try_into().unwrap())
            .unwrap();
          self
            .options
            .chain()
            .address_from_script(&output.script_pubkey)
            .map(|address| address.to_string())
            .unwrap_or_else(|e| e.to_string())
        };
        write!(writer, "\t{}", address)?;
      }
      writeln!(writer)?;

      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break;
      }
    }
    writer.flush()?;
    Ok(())
  }

  fn begin_read(&self) -> Result<rtx::Rtx> {
    Ok(rtx::Rtx(self.database.begin_read()?))
  }

  fn begin_write(&self) -> Result<WriteTransaction> {
    let mut tx = self.database.begin_write()?;
    tx.set_durability(self.durability);
    Ok(tx)
  }

  fn increment_statistic(wtx: &WriteTransaction, statistic: Statistic, n: u64) -> Result {
    let mut statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;
    let value = statistic_to_count
      .get(&(statistic.key()))?
      .map(|x| x.value())
      .unwrap_or_default()
      + n;
    statistic_to_count.insert(&statistic.key(), &value)?;
    Ok(())
  }

  pub(crate) fn set_statistic(
    statistics: &mut Table<u64, u64>,
    statistic: Statistic,
    value: u64,
  ) -> Result<()> {
    statistics.insert(&statistic.key(), &value)?;
    Ok(())
  }

  pub(crate) fn is_statistic_set(
    statistics: &ReadOnlyTable<u64, u64>,
    statistic: Statistic,
  ) -> Result<bool> {
    Ok(
      statistics
        .get(&statistic.key())?
        .map(|guard| guard.value())
        .unwrap_or_default()
        != 0,
    )
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
      .unwrap_or_default()
  }

  pub(crate) fn block_count(&self) -> Result<u32> {
    self.begin_read()?.block_count()
  }

  pub(crate) fn block_height(&self) -> Result<Option<Height>> {
    self.begin_read()?.block_height()
  }

  pub(crate) fn block_hash(&self, height: Option<u32>) -> Result<Option<BlockHash>> {
    self.begin_read()?.block_hash(height)
  }

  pub(crate) fn blocks(&self, take: usize) -> Result<Vec<(u32, BlockHash)>> {
    let rtx = self.begin_read()?;

    let block_count = rtx.block_count()?;

    let height_to_block_header = rtx.0.open_table(HEIGHT_TO_BLOCK_HEADER)?;

    let mut blocks = Vec::with_capacity(block_count.try_into().unwrap());

    for next in height_to_block_header
      .range(0..block_count)?
      .rev()
      .take(take)
    {
      let next = next?;
      blocks.push((next.0.value(), Header::load(*next.1.value()).block_hash()));
    }

    Ok(blocks)
  }

  pub(crate) fn rare_sat_satpoints(&self) -> Result<Vec<(Sat, SatPoint)>> {
    let rtx = self.database.begin_read()?;

    let sat_to_satpoint = rtx.open_table(SAT_TO_SATPOINT)?;

    let mut result = Vec::with_capacity(sat_to_satpoint.len()?.try_into().unwrap());

    for range in sat_to_satpoint.range(0..)? {
      let (sat, satpoint) = range?;
      result.push((Sat(sat.value()), Entry::load(*satpoint.value())));
    }

    Ok(result)
  }

  pub(crate) fn rare_sat_satpoint(&self, sat: Sat) -> Result<Option<SatPoint>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(SAT_TO_SATPOINT)?
        .get(&sat.n())?
        .map(|satpoint| Entry::load(*satpoint.value())),
    )
  }

  pub(crate) fn get_rune_by_id(&self, id: RuneId) -> Result<Option<Rune>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(RUNE_ID_TO_RUNE_ENTRY)?
        .get(&id.store())?
        .map(|entry| RuneEntry::load(entry.value()).rune),
    )
  }

  pub(crate) fn rune(
    &self,
    rune: Rune,
  ) -> Result<Option<(RuneId, RuneEntry, Option<InscriptionId>)>> {
    let rtx = self.database.begin_read()?;

    let Some(id) = rtx
      .open_table(RUNE_TO_RUNE_ID)?
      .get(rune.0)?
      .map(|guard| guard.value())
    else {
      return Ok(None);
    };

    let entry = RuneEntry::load(
      rtx
        .open_table(RUNE_ID_TO_RUNE_ENTRY)?
        .get(id)?
        .unwrap()
        .value(),
    );

    let parent = InscriptionId {
      txid: entry.etching,
      index: 0,
    };

    let parent = rtx
      .open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?
      .get(&parent.store())?
      .is_some()
      .then_some(parent);

    Ok(Some((RuneId::load(id), entry, parent)))
  }

  pub(crate) fn runes(&self) -> Result<Vec<(RuneId, RuneEntry)>> {
    let mut entries = Vec::new();

    for result in self
      .database
      .begin_read()?
      .open_table(RUNE_ID_TO_RUNE_ENTRY)?
      .iter()?
    {
      let (id, entry) = result?;
      entries.push((RuneId::load(id.value()), RuneEntry::load(entry.value())));
    }

    Ok(entries)
  }

  pub(crate) fn get_rune_balance(&self, outpoint: OutPoint, id: RuneId) -> Result<u128> {
    let rtx = self.database.begin_read()?;

    let outpoint_to_balances = rtx.open_table(OUTPOINT_TO_RUNE_BALANCES)?;

    let Some(balances) = outpoint_to_balances.get(&outpoint.store())? else {
      return Ok(0);
    };

    let balances_buffer = balances.value();

    let mut i = 0;
    while i < balances_buffer.len() {
      let (balance_id, length) = runes::varint::decode(&balances_buffer[i..]);
      i += length;
      let (amount, length) = runes::varint::decode(&balances_buffer[i..]);
      i += length;

      if RuneId::try_from(balance_id).unwrap() == id {
        return Ok(amount);
      }
    }

    Ok(0)
  }

  pub(crate) fn get_rune_balances_for_outpoint(
    &self,
    outpoint: OutPoint,
  ) -> Result<Vec<(SpacedRune, Pile)>> {
    let rtx = self.database.begin_read()?;

    let outpoint_to_balances = rtx.open_table(OUTPOINT_TO_RUNE_BALANCES)?;

    let id_to_rune_entries = rtx.open_table(RUNE_ID_TO_RUNE_ENTRY)?;

    let Some(balances) = outpoint_to_balances.get(&outpoint.store())? else {
      return Ok(Vec::new());
    };

    let balances_buffer = balances.value();

    let mut balances = Vec::new();
    let mut i = 0;
    while i < balances_buffer.len() {
      let (id, length) = runes::varint::decode(&balances_buffer[i..]);
      i += length;
      let (amount, length) = runes::varint::decode(&balances_buffer[i..]);
      i += length;

      let id = RuneId::try_from(id).unwrap();

      let entry = RuneEntry::load(id_to_rune_entries.get(id.store())?.unwrap().value());

      balances.push((
        entry.spaced_rune(),
        Pile {
          amount,
          divisibility: entry.divisibility,
          symbol: entry.symbol,
        },
      ));
    }

    Ok(balances)
  }

  pub(crate) fn get_runic_outputs(&self, outpoints: &[OutPoint]) -> Result<BTreeSet<OutPoint>> {
    let rtx = self.database.begin_read()?;

    let outpoint_to_balances = rtx.open_table(OUTPOINT_TO_RUNE_BALANCES)?;

    let mut runic = BTreeSet::new();

    for outpoint in outpoints {
      if outpoint_to_balances.get(&outpoint.store())?.is_some() {
        runic.insert(*outpoint);
      }
    }

    Ok(runic)
  }

  pub(crate) fn get_rune_balance_map(&self) -> Result<BTreeMap<Rune, BTreeMap<OutPoint, u128>>> {
    let outpoint_balances = self.get_rune_balances()?;

    let rtx = self.database.begin_read()?;

    let rune_id_to_rune_entry = rtx.open_table(RUNE_ID_TO_RUNE_ENTRY)?;

    let mut rune_balances: BTreeMap<Rune, BTreeMap<OutPoint, u128>> = BTreeMap::new();

    for (outpoint, balances) in outpoint_balances {
      for (rune_id, amount) in balances {
        let rune = RuneEntry::load(
          rune_id_to_rune_entry
            .get(&rune_id.store())?
            .unwrap()
            .value(),
        )
        .rune;

        *rune_balances
          .entry(rune)
          .or_default()
          .entry(outpoint)
          .or_default() += amount;
      }
    }

    Ok(rune_balances)
  }

  pub(crate) fn get_rune_balances(&self) -> Result<Vec<(OutPoint, Vec<(RuneId, u128)>)>> {
    let mut result = Vec::new();

    for entry in self
      .database
      .begin_read()?
      .open_table(OUTPOINT_TO_RUNE_BALANCES)?
      .iter()?
    {
      let (outpoint, balances_buffer) = entry?;
      let outpoint = OutPoint::load(*outpoint.value());
      let balances_buffer = balances_buffer.value();

      let mut balances = Vec::new();
      let mut i = 0;
      while i < balances_buffer.len() {
        let (id, length) = runes::varint::decode(&balances_buffer[i..]);
        i += length;
        let (balance, length) = runes::varint::decode(&balances_buffer[i..]);
        i += length;
        balances.push((RuneId::try_from(id)?, balance));
      }

      result.push((outpoint, balances));
    }

    Ok(result)
  }

  pub(crate) fn block_header(&self, hash: BlockHash) -> Result<Option<Header>> {
    self.client.get_block_header(&hash).into_option()
  }

  pub(crate) fn block_header_info(&self, hash: BlockHash) -> Result<Option<GetBlockHeaderResult>> {
    self.client.get_block_header_info(&hash).into_option()
  }

  pub(crate) fn get_block_by_height(&self, height: u32) -> Result<Option<Block>> {
    Ok(
      self
        .client
        .get_block_hash(height.into())
        .into_option()?
        .map(|hash| self.client.get_block(&hash))
        .transpose()?,
    )
  }

  pub(crate) fn get_block_by_hash(&self, hash: BlockHash) -> Result<Option<Block>> {
    self.client.get_block(&hash).into_option()
  }

  pub(crate) fn get_collections_paginated(
    &self,
    page_size: usize,
    page_index: usize,
  ) -> Result<(Vec<InscriptionId>, bool)> {
    let rtx = self.database.begin_read()?;

    let sequence_number_to_inscription_entry =
      rtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

    let mut collections = rtx
      .open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN)?
      .iter()?
      .skip(page_index.saturating_mul(page_size))
      .take(page_size.saturating_add(1))
      .map(|result| {
        result
          .and_then(|(parent, _children)| {
            sequence_number_to_inscription_entry
              .get(parent.value())
              .map(|entry| InscriptionEntry::load(entry.unwrap().value()).id)
          })
          .map_err(|err| err.into())
      })
      .collect::<Result<Vec<InscriptionId>>>()?;

    let more = collections.len() > page_size;

    if more {
      collections.pop();
    }

    Ok((collections, more))
  }

  #[cfg(test)]
  pub(crate) fn get_children_by_inscription_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Vec<InscriptionId>> {
    let rtx = self.database.begin_read()?;

    let Some(sequence_number) = rtx
      .open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?
      .get(&inscription_id.store())?
      .map(|sequence_number| sequence_number.value())
    else {
      return Ok(Vec::new());
    };

    self
      .get_children_by_sequence_number_paginated(sequence_number, usize::max_value(), 0)
      .map(|(children, _more)| children)
  }

  #[cfg(test)]
  pub(crate) fn get_parent_by_inscription_id(
    &self,
    inscription_id: InscriptionId,
  ) -> InscriptionId {
    let rtx = self.database.begin_read().unwrap();

    let sequence_number = rtx
      .open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)
      .unwrap()
      .get(&inscription_id.store())
      .unwrap()
      .unwrap()
      .value();

    let sequence_number_to_inscription_entry = rtx
      .open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)
      .unwrap();

    let parent_sequence_number = InscriptionEntry::load(
      sequence_number_to_inscription_entry
        .get(sequence_number)
        .unwrap()
        .unwrap()
        .value(),
    )
    .parent
    .unwrap();

    let entry = InscriptionEntry::load(
      sequence_number_to_inscription_entry
        .get(parent_sequence_number)
        .unwrap()
        .unwrap()
        .value(),
    );

    entry.id
  }

  pub(crate) fn get_children_by_sequence_number_paginated(
    &self,
    sequence_number: u32,
    page_size: usize,
    page_index: usize,
  ) -> Result<(Vec<InscriptionId>, bool)> {
    let rtx = self.database.begin_read()?;

    let sequence_number_to_entry = rtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

    let mut children = rtx
      .open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN)?
      .get(sequence_number)?
      .skip(page_index * page_size)
      .take(page_size.saturating_add(1))
      .map(|result| {
        result
          .and_then(|sequence_number| {
            sequence_number_to_entry
              .get(sequence_number.value())
              .map(|entry| InscriptionEntry::load(entry.unwrap().value()).id)
          })
          .map_err(|err| err.into())
      })
      .collect::<Result<Vec<InscriptionId>>>()?;

    let more = children.len() > page_size;

    if more {
      children.pop();
    }

    Ok((children, more))
  }

  pub(crate) fn get_etching(&self, txid: Txid) -> Result<Option<SpacedRune>> {
    let rtx = self.database.begin_read()?;

    let transaction_id_to_rune = rtx.open_table(TRANSACTION_ID_TO_RUNE)?;
    let Some(rune) = transaction_id_to_rune.get(&txid.store())? else {
      return Ok(None);
    };

    let rune_to_rune_id = rtx.open_table(RUNE_TO_RUNE_ID)?;
    let id = rune_to_rune_id.get(rune.value())?.unwrap();

    let rune_id_to_rune_entry = rtx.open_table(RUNE_ID_TO_RUNE_ENTRY)?;
    let entry = rune_id_to_rune_entry.get(&id.value())?.unwrap();

    Ok(Some(RuneEntry::load(entry.value()).spaced_rune()))
  }

  pub(crate) fn get_inscription_ids_by_sat(&self, sat: Sat) -> Result<Vec<InscriptionId>> {
    let rtx = self.database.begin_read()?;

    let sequence_number_to_inscription_entry =
      rtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

    let ids = rtx
      .open_multimap_table(SAT_TO_SEQUENCE_NUMBER)?
      .get(&sat.n())?
      .map(|result| {
        result
          .and_then(|sequence_number| {
            let sequence_number = sequence_number.value();
            sequence_number_to_inscription_entry
              .get(sequence_number)
              .map(|entry| InscriptionEntry::load(entry.unwrap().value()).id)
          })
          .map_err(|err| err.into())
      })
      .collect::<Result<Vec<InscriptionId>>>()?;

    Ok(ids)
  }

  pub(crate) fn get_inscription_ids_by_sat_paginated(
    &self,
    sat: Sat,
    page_size: u64,
    page_index: u64,
  ) -> Result<(Vec<InscriptionId>, bool)> {
    let rtx = self.database.begin_read()?;

    let sequence_number_to_inscription_entry =
      rtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

    let mut ids = rtx
      .open_multimap_table(SAT_TO_SEQUENCE_NUMBER)?
      .get(&sat.n())?
      .map(|result| {
        result
          .and_then(|sequence_number| {
            let sequence_number = sequence_number.value();
            sequence_number_to_inscription_entry
              .get(sequence_number)
              .map(|entry| InscriptionEntry::load(entry.unwrap().value()).id)
          })
          .map_err(|err| err.into())
      })
      .skip(page_index.saturating_mul(page_size).try_into().unwrap())
      .take(page_size.saturating_add(1).try_into().unwrap())
      .collect::<Result<Vec<InscriptionId>>>()?;

    let more = ids.len() > page_size.try_into().unwrap();

    if more {
      ids.pop();
    }

    Ok((ids, more))
  }

  pub(crate) fn get_inscription_id_by_sat_indexed(
    &self,
    sat: Sat,
    inscription_index: isize,
  ) -> Result<Option<InscriptionId>> {
    let rtx = self.database.begin_read()?;

    let sequence_number_to_inscription_entry =
      rtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

    let sat_to_sequence_number = rtx.open_multimap_table(SAT_TO_SEQUENCE_NUMBER)?;

    if inscription_index < 0 {
      sat_to_sequence_number
        .get(&sat.n())?
        .nth_back((inscription_index + 1).abs_diff(0))
    } else {
      sat_to_sequence_number
        .get(&sat.n())?
        .nth(inscription_index.abs_diff(0))
    }
    .map(|result| {
      result
        .and_then(|sequence_number| {
          let sequence_number = sequence_number.value();
          sequence_number_to_inscription_entry
            .get(sequence_number)
            .map(|entry| InscriptionEntry::load(entry.unwrap().value()).id)
        })
        .map_err(|err| anyhow!(err.to_string()))
    })
    .transpose()
  }

  #[cfg(test)]
  pub(crate) fn get_inscription_id_by_inscription_number(
    &self,
    inscription_number: i32,
  ) -> Result<Option<InscriptionId>> {
    let rtx = self.database.begin_read()?;

    let Some(sequence_number) = rtx
      .open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER)?
      .get(inscription_number)?
      .map(|guard| guard.value())
    else {
      return Ok(None);
    };

    let inscription_id = rtx
      .open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?
      .get(&sequence_number)?
      .map(|entry| InscriptionEntry::load(entry.value()).id);

    Ok(inscription_id)
  }

  pub(crate) fn get_inscription_satpoint_by_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<SatPoint>> {
    let rtx = self.database.begin_read()?;

    let Some(sequence_number) = rtx
      .open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?
      .get(&inscription_id.store())?
      .map(|guard| guard.value())
    else {
      return Ok(None);
    };

    let satpoint = rtx
      .open_table(SEQUENCE_NUMBER_TO_SATPOINT)?
      .get(sequence_number)?
      .map(|satpoint| Entry::load(*satpoint.value()));

    Ok(satpoint)
  }

  pub(crate) fn get_inscription_by_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<Inscription>> {
    if !self.inscription_exists(inscription_id)? {
      return Ok(None);
    }

    Ok(self.get_transaction(inscription_id.txid)?.and_then(|tx| {
      ParsedEnvelope::from_transaction(&tx)
        .into_iter()
        .nth(inscription_id.index as usize)
        .map(|envelope| envelope.payload)
    }))
  }

  pub(crate) fn inscription_count(&self, txid: Txid) -> Result<u32> {
    let start = InscriptionId { index: 0, txid };

    let end = InscriptionId {
      index: u32::MAX,
      txid,
    };

    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?
        .range::<&InscriptionIdValue>(&start.store()..&end.store())?
        .count()
        .try_into()
        .unwrap(),
    )
  }

  pub(crate) fn inscription_exists(&self, inscription_id: InscriptionId) -> Result<bool> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?
        .get(&inscription_id.store())?
        .is_some(),
    )
  }

  pub(crate) fn get_inscriptions_on_output_with_satpoints(
    &self,
    outpoint: OutPoint,
  ) -> Result<Vec<(SatPoint, InscriptionId)>> {
    let rtx = self.database.begin_read()?;
    let satpoint_to_sequence_number = rtx.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER)?;
    let sequence_number_to_inscription_entry =
      rtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

    Self::inscriptions_on_output(
      &satpoint_to_sequence_number,
      &sequence_number_to_inscription_entry,
      outpoint,
    )
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
      return Ok(Some(self.genesis_block_coinbase_transaction.clone()));
    }

    if self.index_transactions {
      if let Some(transaction) = self
        .database
        .begin_read()?
        .open_table(TRANSACTION_ID_TO_TRANSACTION)?
        .get(&txid.store())?
      {
        return Ok(Some(consensus::encode::deserialize(transaction.value())?));
      }
    }

    self.client.get_raw_transaction(&txid, None).into_option()
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

  pub(crate) fn find(&self, sat: Sat) -> Result<Option<SatPoint>> {
    let sat = sat.0;
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

  pub(crate) fn find_range(
    &self,
    range_start: Sat,
    range_end: Sat,
  ) -> Result<Option<Vec<FindRangeOutput>>> {
    let range_start = range_start.0;
    let range_end = range_end.0;
    let rtx = self.begin_read()?;

    if rtx.block_count()? < Sat(range_end - 1).height().n() + 1 {
      return Ok(None);
    }

    let Some(mut remaining_sats) = range_end.checked_sub(range_start) else {
      return Err(anyhow!("range end is before range start"));
    };

    let outpoint_to_sat_ranges = rtx.0.open_table(OUTPOINT_TO_SAT_RANGES)?;

    let mut result = Vec::new();
    for range in outpoint_to_sat_ranges.range::<&[u8; 36]>(&[0; 36]..)? {
      let (outpoint_entry, sat_ranges_entry) = range?;

      let mut offset = 0;
      for sat_range in sat_ranges_entry.value().chunks_exact(11) {
        let (start, end) = SatRange::load(sat_range.try_into().unwrap());

        if end > range_start && start < range_end {
          let overlap_start = start.max(range_start);
          let overlap_end = end.min(range_end);

          result.push(FindRangeOutput {
            start: overlap_start,
            size: overlap_end - overlap_start,
            satpoint: SatPoint {
              outpoint: Entry::load(*outpoint_entry.value()),
              offset: offset + overlap_start - start,
            },
          });

          remaining_sats -= overlap_end - overlap_start;

          if remaining_sats == 0 {
            break;
          }
        }
        offset += end - start;
      }
    }

    Ok(Some(result))
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
    if !self.index_sats || outpoint == unbound_outpoint() {
      return Ok(None);
    }

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

    let rtx = self.database.begin_read()?;

    let height_to_block_header = rtx.open_table(HEIGHT_TO_BLOCK_HEADER)?;

    if let Some(guard) = height_to_block_header.get(height)? {
      return Ok(Blocktime::confirmed(Header::load(*guard.value()).time));
    }

    let current = height_to_block_header
      .range(0..)?
      .next_back()
      .and_then(|result| result.ok())
      .map(|(height, _header)| height)
      .map(|x| x.value())
      .unwrap_or(0);

    let expected_blocks = height
      .checked_sub(current)
      .with_context(|| format!("current {current} height is greater than sat height {height}"))?;

    Ok(Blocktime::Expected(
      Utc::now()
        .round_subsecs(0)
        .checked_add_signed(chrono::Duration::seconds(
          10 * 60 * i64::from(expected_blocks),
        ))
        .ok_or_else(|| anyhow!("block timestamp out of range"))?,
    ))
  }

  pub(crate) fn get_inscriptions(
    &self,
    utxos: &BTreeMap<OutPoint, Amount>,
  ) -> Result<BTreeMap<SatPoint, InscriptionId>> {
    let rtx = self.database.begin_read()?;

    let mut result = BTreeMap::new();

    let satpoint_to_sequence_number = rtx.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER)?;
    let sequence_number_to_inscription_entry =
      rtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;
    for utxo in utxos.keys() {
      result.extend(Self::inscriptions_on_output(
        &satpoint_to_sequence_number,
        &sequence_number_to_inscription_entry,
        *utxo,
      )?);
    }

    Ok(result)
  }

  pub(crate) fn get_inscriptions_paginated(
    &self,
    page_size: u32,
    page_index: u32,
  ) -> Result<(Vec<InscriptionId>, bool)> {
    let rtx = self.database.begin_read()?;

    let sequence_number_to_inscription_entry =
      rtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

    let last = sequence_number_to_inscription_entry
      .iter()?
      .next_back()
      .map(|result| result.map(|(number, _entry)| number.value()))
      .transpose()?
      .unwrap_or_default();

    let start = last.saturating_sub(page_size.saturating_mul(page_index));

    let end = start.saturating_sub(page_size);

    let mut inscriptions = sequence_number_to_inscription_entry
      .range(end..=start)?
      .rev()
      .map(|result| result.map(|(_number, entry)| InscriptionEntry::load(entry.value()).id))
      .collect::<Result<Vec<InscriptionId>, StorageError>>()?;

    let more = u32::try_from(inscriptions.len()).unwrap_or(u32::MAX) > page_size;

    if more {
      inscriptions.pop();
    }

    Ok((inscriptions, more))
  }

  pub(crate) fn get_inscriptions_in_block(&self, block_height: u32) -> Result<Vec<InscriptionId>> {
    let rtx = self.database.begin_read()?;

    let height_to_last_sequence_number = rtx.open_table(HEIGHT_TO_LAST_SEQUENCE_NUMBER)?;
    let sequence_number_to_inscription_entry =
      rtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

    let Some(newest_sequence_number) = height_to_last_sequence_number
      .get(&block_height)?
      .map(|ag| ag.value())
    else {
      return Ok(Vec::new());
    };

    let oldest_sequence_number = height_to_last_sequence_number
      .get(block_height.saturating_sub(1))?
      .map(|ag| ag.value())
      .unwrap_or(0);

    (oldest_sequence_number..newest_sequence_number)
      .map(|num| match sequence_number_to_inscription_entry.get(&num) {
        Ok(Some(inscription_id)) => Ok(InscriptionEntry::load(inscription_id.value()).id),
        Ok(None) => Err(anyhow!(
          "could not find inscription for inscription number {num}"
        )),
        Err(err) => Err(anyhow!(err)),
      })
      .collect::<Result<Vec<InscriptionId>>>()
  }

  pub(crate) fn get_highest_paying_inscriptions_in_block(
    &self,
    block_height: u32,
    n: usize,
  ) -> Result<(Vec<InscriptionId>, usize)> {
    let inscription_ids = self.get_inscriptions_in_block(block_height)?;

    let mut inscription_to_fee: Vec<(InscriptionId, u64)> = Vec::new();
    for id in &inscription_ids {
      inscription_to_fee.push((
        *id,
        self
          .get_inscription_entry(*id)?
          .ok_or_else(|| anyhow!("could not get entry for inscription {id}"))?
          .fee,
      ));
    }

    inscription_to_fee.sort_by_key(|(_, fee)| *fee);

    Ok((
      inscription_to_fee
        .iter()
        .map(|(id, _)| *id)
        .rev()
        .take(n)
        .collect(),
      inscription_ids.len(),
    ))
  }

  pub(crate) fn get_home_inscriptions(&self) -> Result<Vec<InscriptionId>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(HOME_INSCRIPTIONS)?
        .iter()?
        .rev()
        .flat_map(|result| result.map(|(_number, id)| InscriptionId::load(id.value())))
        .collect(),
    )
  }

  pub(crate) fn get_feed_inscriptions(&self, n: usize) -> Result<Vec<(u32, InscriptionId)>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?
        .iter()?
        .rev()
        .take(n)
        .flat_map(|result| {
          result.map(|(number, entry)| (number.value(), InscriptionEntry::load(entry.value()).id))
        })
        .collect(),
    )
  }

  pub fn inscription_info_benchmark(index: &Index, inscription_number: i32) {
    Self::inscription_info(index, InscriptionQuery::Number(inscription_number)).unwrap();
  }

  pub(crate) fn inscription_info(
    index: &Index,
    query: InscriptionQuery,
  ) -> Result<Option<InscriptionInfo>> {
    let rtx = index.database.begin_read()?;

    let sequence_number = match query {
      InscriptionQuery::Id(id) => {
        let inscription_id_to_sequence_number =
          rtx.open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;

        let sequence_number = inscription_id_to_sequence_number
          .get(&id.store())?
          .map(|guard| guard.value());

        drop(inscription_id_to_sequence_number);

        sequence_number
      }
      InscriptionQuery::Number(inscription_number) => {
        let inscription_number_to_sequence_number =
          rtx.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER)?;

        let sequence_number = inscription_number_to_sequence_number
          .get(inscription_number)?
          .map(|guard| guard.value());

        drop(inscription_number_to_sequence_number);

        sequence_number
      }
    };

    let Some(sequence_number) = sequence_number else {
      return Ok(None);
    };

    let sequence_number_to_inscription_entry =
      rtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

    let entry = InscriptionEntry::load(
      sequence_number_to_inscription_entry
        .get(&sequence_number)?
        .unwrap()
        .value(),
    );

    let Some(transaction) = index.get_transaction(entry.id.txid)? else {
      return Ok(None);
    };

    let Some(inscription) = ParsedEnvelope::from_transaction(&transaction)
      .into_iter()
      .nth(entry.id.index as usize)
      .map(|envelope| envelope.payload)
    else {
      return Ok(None);
    };

    let satpoint = SatPoint::load(
      *rtx
        .open_table(SEQUENCE_NUMBER_TO_SATPOINT)?
        .get(sequence_number)?
        .unwrap()
        .value(),
    );

    let output = if satpoint.outpoint == unbound_outpoint() || satpoint.outpoint == OutPoint::null()
    {
      None
    } else {
      let Some(transaction) = index.get_transaction(satpoint.outpoint.txid)? else {
        return Ok(None);
      };

      transaction
        .output
        .into_iter()
        .nth(satpoint.outpoint.vout.try_into().unwrap())
    };

    let previous = if let Some(n) = sequence_number.checked_sub(1) {
      Some(
        InscriptionEntry::load(
          sequence_number_to_inscription_entry
            .get(n)?
            .unwrap()
            .value(),
        )
        .id,
      )
    } else {
      None
    };

    let next = sequence_number_to_inscription_entry
      .get(sequence_number + 1)?
      .map(|guard| InscriptionEntry::load(guard.value()).id);

    let children = rtx
      .open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN)?
      .get(sequence_number)?
      .take(4)
      .map(|result| {
        result
          .and_then(|sequence_number| {
            sequence_number_to_inscription_entry
              .get(sequence_number.value())
              .map(|entry| InscriptionEntry::load(entry.unwrap().value()).id)
          })
          .map_err(|err| err.into())
      })
      .collect::<Result<Vec<InscriptionId>>>()?;

    let rune = if let Some(rune_id) = rtx
      .open_table(SEQUENCE_NUMBER_TO_RUNE_ID)?
      .get(sequence_number)?
    {
      let rune_id_to_rune_entry = rtx.open_table(RUNE_ID_TO_RUNE_ENTRY)?;
      let entry = rune_id_to_rune_entry.get(&rune_id.value())?.unwrap();
      Some(RuneEntry::load(entry.value()).spaced_rune())
    } else {
      None
    };

    let parent = match entry.parent {
      Some(parent) => Some(
        InscriptionEntry::load(
          sequence_number_to_inscription_entry
            .get(parent)?
            .unwrap()
            .value(),
        )
        .id,
      ),
      None => None,
    };

    let mut charms = entry.charms;

    if satpoint.outpoint == OutPoint::null() {
      Charm::Lost.set(&mut charms);
    }

    Ok(Some(InscriptionInfo {
      children,
      entry,
      parent,
      output,
      satpoint,
      inscription,
      previous,
      next,
      rune,
      charms,
    }))
  }

  pub(crate) fn get_inscription_entry(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<InscriptionEntry>> {
    let rtx = self.database.begin_read()?;

    let Some(sequence_number) = rtx
      .open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?
      .get(&inscription_id.store())?
      .map(|guard| guard.value())
    else {
      return Ok(None);
    };

    let entry = rtx
      .open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?
      .get(sequence_number)?
      .map(|value| InscriptionEntry::load(value.value()));

    Ok(entry)
  }

  #[cfg(test)]
  fn assert_inscription_location(
    &self,
    inscription_id: InscriptionId,
    satpoint: SatPoint,
    sat: Option<u64>,
  ) {
    let rtx = self.database.begin_read().unwrap();

    let satpoint_to_sequence_number = rtx
      .open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER)
      .unwrap();

    let sequence_number_to_satpoint = rtx.open_table(SEQUENCE_NUMBER_TO_SATPOINT).unwrap();

    let sequence_number = rtx
      .open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)
      .unwrap()
      .get(&inscription_id.store())
      .unwrap()
      .unwrap()
      .value();

    assert_eq!(
      satpoint_to_sequence_number.len().unwrap(),
      sequence_number_to_satpoint.len().unwrap(),
    );

    assert_eq!(
      SatPoint::load(
        *sequence_number_to_satpoint
          .get(sequence_number)
          .unwrap()
          .unwrap()
          .value()
      ),
      satpoint,
    );

    assert!(satpoint_to_sequence_number
      .get(&satpoint.store())
      .unwrap()
      .any(|result| result.unwrap().value() == sequence_number));

    match sat {
      Some(sat) => {
        if self.index_sats {
          // unbound inscriptions should not be assigned to a sat
          assert!(satpoint.outpoint != unbound_outpoint());

          assert!(rtx
            .open_multimap_table(SAT_TO_SEQUENCE_NUMBER)
            .unwrap()
            .get(&sat)
            .unwrap()
            .any(|entry| entry.unwrap().value() == sequence_number));

          // we do not track common sats (only the sat ranges)
          if !Sat(sat).common() {
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
        if self.index_sats {
          assert!(satpoint.outpoint == unbound_outpoint())
        }
      }
    }
  }

  fn inscriptions_on_output<'a: 'tx, 'tx>(
    satpoint_to_sequence_number: &'a impl ReadableMultimapTable<&'static SatPointValue, u32>,
    sequence_number_to_inscription_entry: &'a impl ReadableTable<u32, InscriptionEntryValue>,
    outpoint: OutPoint,
  ) -> Result<Vec<(SatPoint, InscriptionId)>> {
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

    for range in satpoint_to_sequence_number.range::<&[u8; 44]>(&start..=&end)? {
      let (satpoint, sequence_numbers) = range?;
      for sequence_number_result in sequence_numbers {
        let sequence_number = sequence_number_result?.value();
        let entry = sequence_number_to_inscription_entry
          .get(sequence_number)?
          .unwrap();
        inscriptions.push((
          sequence_number,
          SatPoint::load(*satpoint.value()),
          InscriptionEntry::load(entry.value()).id,
        ));
      }
    }

    inscriptions.sort_by_key(|(sequence_number, _, _)| *sequence_number);

    Ok(
      inscriptions
        .into_iter()
        .map(|(_sequence_number, satpoint, inscription_id)| (satpoint, inscription_id))
        .collect(),
    )
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    crate::index::testing::Context,
    bitcoin::secp256k1::rand::{self, RngCore},
  };

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
      inputs: &[(1, 0, 0, inscription.to_witness())],
      ..Default::default()
    };

    {
      let context = Context::builder().build();
      context.mine_blocks(1);
      let txid = context.rpc_server.broadcast_tx(template.clone());
      let inscription_id = InscriptionId { txid, index: 0 };
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
      let inscription_id = InscriptionId { txid, index: 0 };
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
  fn inscriptions_are_not_indexed_if_no_index_inscriptions_flag_is_set() {
    let inscription = inscription("text/plain;charset=utf-8", "hello");
    let template = TransactionTemplate {
      inputs: &[(1, 0, 0, inscription.to_witness())],
      ..Default::default()
    };

    {
      let context = Context::builder().build();
      context.mine_blocks(1);
      let txid = context.rpc_server.broadcast_tx(template.clone());
      let inscription_id = InscriptionId { txid, index: 0 };
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
      let context = Context::builder().arg("--no-index-inscriptions").build();
      context.mine_blocks(1);
      let txid = context.rpc_server.broadcast_tx(template);
      let inscription_id = InscriptionId { txid, index: 0 };
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
      inputs: &[(1, 0, 0, Default::default())],
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
      inputs: &[(1, 0, 0, Default::default()), (2, 0, 0, Default::default())],
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
      inputs: &[(1, 0, 0, Default::default())],
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
      inputs: &[(1, 0, 0, Default::default())],
      fee: 10,
      ..Default::default()
    };
    let second_fee_paying_tx = TransactionTemplate {
      inputs: &[(2, 0, 0, Default::default())],
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
      inputs: &[(1, 0, 0, Default::default())],
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
      inputs: &[(1, 0, 0, Default::default())],
      fee: 50 * COIN_VALUE,
      ..Default::default()
    };
    context.rpc_server.broadcast_tx(no_value_output);
    context.mine_blocks(1);

    let no_value_input = TransactionTemplate {
      inputs: &[(2, 1, 0, Default::default())],
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
      inputs: &[(1, 0, 0, Default::default())],
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
      context.index.find(Sat(0)).unwrap().unwrap(),
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
      context.index.find(Sat(1)).unwrap().unwrap(),
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
      context.index.find(Sat(50 * COIN_VALUE)).unwrap().unwrap(),
      SatPoint {
        outpoint: "84aca0d43f45ac753d4744f40b2f54edec3a496b298951735d450e601386089d:0"
          .parse()
          .unwrap(),
        offset: 0,
      }
    )
  }

  #[test]
  fn find_unmined_sat() {
    let context = Context::builder().arg("--index-sats").build();
    assert_eq!(context.index.find(Sat(50 * COIN_VALUE)).unwrap(), None);
  }

  #[test]
  fn find_first_sat_spent_in_second_block() {
    let context = Context::builder().arg("--index-sats").build();
    context.mine_blocks(1);
    let spend_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Default::default())],
      fee: 0,
      ..Default::default()
    });
    context.mine_blocks(1);
    assert_eq!(
      context.index.find(Sat(50 * COIN_VALUE)).unwrap().unwrap(),
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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

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
        inputs: &[(1, 0, 0, Default::default())],
        fee: 50 * 100_000_000,
        ..Default::default()
      });

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

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
        inputs: &[(4, 0, 0, Default::default())],
        fee: 50 * 100_000_000,
        ..Default::default()
      });

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(5, 1, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

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
        inputs: &[(2, 0, 0, Default::default()), (2, 1, 0, Default::default())],
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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let first_inscription_id = InscriptionId {
        txid: first_txid,
        index: 0,
      };

      let second_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0, inscription("text/png", [1; 100]).to_witness())],
        ..Default::default()
      });
      let second_inscription_id = InscriptionId {
        txid: second_txid,
        index: 0,
      };

      context.mine_blocks(1);

      let merged_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 0, Default::default()), (3, 2, 0, Default::default())],
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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

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
        inputs: &[(2, 0, 0, Default::default()), (2, 1, 0, Default::default())],
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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

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
        inputs: &[(2, 0, 0, Default::default()), (2, 1, 0, Default::default())],
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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0, Default::default())],
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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0, Default::default()), (3, 1, 0, Default::default())],
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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      let first_inscription_id = InscriptionId {
        txid: first_txid,
        index: 0,
      };

      context.mine_blocks_with_subsidy(1, 0);
      context.mine_blocks(1);

      let second_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 0, 0, inscription("text/plain", "hello").to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      let second_inscription_id = InscriptionId {
        txid: second_txid,
        index: 0,
      };

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
    let context = Context::builder()
      .args(["--index-sats", "--first-inscription-height", "10"])
      .build();
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
    let context = Context::builder()
      .args(["--index-sats", "--first-inscription-height", "10"])
      .build();

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
        inputs: &[(2, 0, 0, inscription("text/plain", "hello").to_witness())],
        outputs: 2,
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };
      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 1, Default::default()), (3, 1, 0, Default::default())],
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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        outputs: 2,
        output_values: &[0, 50 * COIN_VALUE],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };
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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };
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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

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
        inputs: &[(2, 1, 0, Default::default())],
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
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId {
        txid: first,
        index: 0,
      };

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
        inputs: &[(2, 1, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let inscription_id = InscriptionId {
        txid: second,
        index: 0,
      };

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
        .get_inscription_by_id(InscriptionId {
          txid: second,
          index: 0
        })
        .unwrap()
        .is_some());

      assert!(context
        .index
        .get_inscription_by_id(InscriptionId {
          txid: second,
          index: 0
        })
        .unwrap()
        .is_some());
    }
  }

  #[test]
  fn get_latest_inscriptions_with_no_more() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });
      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      let (inscriptions, more) = context.index.get_inscriptions_paginated(100, 0).unwrap();
      assert_eq!(inscriptions, &[inscription_id]);
      assert!(!more);
    }
  }

  #[test]
  fn get_latest_inscriptions_with_more() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let mut ids = Vec::new();

      for i in 0..101 {
        let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
          inputs: &[(i + 1, 0, 0, inscription("text/plain", "hello").to_witness())],
          ..Default::default()
        });
        context.mine_blocks(1);
        ids.push(InscriptionId { txid, index: 0 });
      }

      ids.reverse();
      ids.pop();

      assert_eq!(ids.len(), 100);

      let (inscriptions, more) = context.index.get_inscriptions_paginated(100, 0).unwrap();
      assert_eq!(inscriptions, ids);
      assert!(more);
    }
  }

  #[test]
  fn unsynced_index_fails() {
    for context in Context::configurations() {
      let mut entropy = [0; 16];
      rand::thread_rng().fill_bytes(&mut entropy);
      let mnemonic = Mnemonic::from_entropy(&entropy).unwrap();
      crate::subcommand::wallet::initialize("ord".into(), &context.options, mnemonic.to_seed(""))
        .unwrap();
      context.rpc_server.mine_blocks(1);
      assert_regex_match!(
        crate::subcommand::wallet::get_unspent_outputs(
          &crate::subcommand::wallet::bitcoin_rpc_client_for_wallet_command(
            "ord".to_string(),
            &context.options,
          )
          .unwrap(),
          &context.index
        )
        .unwrap_err()
        .to_string(),
        r"output in Bitcoin Core wallet but not in ord index: [[:xdigit:]]{64}:\d+"
      );
    }
  }

  #[test]
  fn unrecognized_even_field_inscriptions_are_cursed_and_unbound() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let witness = envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[2],
        b"bar",
        &[4],
        b"ord",
      ]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

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
          .inscription_number,
        -1
      );
    }
  }

  #[test]
  fn unrecognized_even_field_inscriptions_are_unbound_after_jubilee() {
    for context in Context::configurations() {
      context.mine_blocks(109);

      let witness = envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[2],
        b"bar",
        &[4],
        b"ord",
      ]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

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
          .inscription_number,
        0
      );
    }
  }

  #[test]
  fn inscriptions_are_uncursed_after_jubilee() {
    for context in Context::configurations() {
      context.mine_blocks(108);

      let witness = envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[1],
        b"text/plain;charset=utf-8",
      ]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness.clone())],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      assert_eq!(context.rpc_server.height(), 109);

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .inscription_number,
        -1
      );

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0, witness)],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      assert_eq!(context.rpc_server.height(), 110);

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .inscription_number,
        0
      );
    }
  }

  #[test]
  fn duplicate_field_inscriptions_are_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let witness = envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[1],
        b"text/plain;charset=utf-8",
      ]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .inscription_number,
        -1
      );
    }
  }

  #[test]
  fn incomplete_field_inscriptions_are_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let witness = envelope(&[b"ord", &[1]]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .inscription_number,
        -1
      );
    }
  }

  #[test]
  fn inscriptions_with_pushnum_opcodes_are_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let script = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([])
        .push_opcode(opcodes::all::OP_PUSHNUM_1)
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_slice(&[script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .inscription_number,
        -1
      );
    }
  }

  #[test]
  fn inscriptions_with_stutter_are_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let script = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([])
        .push_opcode(opcodes::all::OP_PUSHNUM_1)
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_slice(&[script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .inscription_number,
        -1
      );
    }
  }

  // https://github.com/ordinals/ord/issues/2062
  #[test]
  fn zero_value_transaction_inscription_not_cursed_but_unbound() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, Default::default())],
        fee: 50 * 100_000_000,
        ..Default::default()
      });

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

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
          .inscription_number,
        0
      );
    }
  }

  #[test]
  fn transaction_with_inscription_inside_zero_value_2nd_input_should_be_unbound_and_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      // create zero value input
      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, Default::default())],
        fee: 50 * 100_000_000,
        ..Default::default()
      });

      context.mine_blocks(1);

      let witness = inscription("text/plain", "hello").to_witness();

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 0, 0, witness.clone()), (2, 1, 0, witness.clone())],
        ..Default::default()
      });

      let second_inscription_id = InscriptionId { txid, index: 1 };

      context.mine_blocks(1);

      context.index.assert_inscription_location(
        second_inscription_id,
        SatPoint {
          outpoint: unbound_outpoint(),
          offset: 0,
        },
        None,
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second_inscription_id)
          .unwrap()
          .unwrap()
          .inscription_number,
        -1
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
        inputs: &[
          (1, 0, 0, witness.clone()),
          (2, 0, 0, witness.clone()),
          (3, 0, 0, witness.clone()),
        ],
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
          .inscription_number,
        0
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second)
          .unwrap()
          .unwrap()
          .inscription_number,
        -1
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(third)
          .unwrap()
          .unwrap()
          .inscription_number,
        -2
      );
    }
  }

  #[test]
  fn multiple_inscriptions_same_input_are_cursed_reinscriptions() {
    for context in Context::configurations() {
      context.rpc_server.mine_blocks(1);

      let script = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"foo")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"bar")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"qix")
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_slice(&[script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
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
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        third,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(first)
          .unwrap()
          .unwrap()
          .inscription_number,
        0
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(second)
          .unwrap()
          .unwrap()
          .inscription_number,
        -1
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(third)
          .unwrap()
          .unwrap()
          .inscription_number,
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
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"foo")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"bar")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"qix")
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_slice(&[script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[
          (1, 0, 0, witness.clone()),
          (2, 0, 0, witness.clone()),
          (3, 0, 0, witness.clone()),
        ],
        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 }; // normal
      let second = InscriptionId { txid, index: 1 }; // cursed reinscription
      let fourth = InscriptionId { txid, index: 3 }; // cursed but bound
      let ninth = InscriptionId { txid, index: 8 }; // cursed reinscription

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
          .inscription_number,
        0
      );

      assert_eq!(
        context
          .index
          .get_inscription_id_by_inscription_number(-3)
          .unwrap()
          .unwrap(),
        fourth
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(fourth)
          .unwrap()
          .unwrap()
          .inscription_number,
        -3
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(ninth)
          .unwrap()
          .unwrap()
          .inscription_number,
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
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"foo")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"bar")
        .push_opcode(opcodes::all::OP_ENDIF)
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([1])
        .push_slice(b"text/plain;charset=utf-8")
        .push_slice([])
        .push_slice(b"qix")
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_slice(&[script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
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
        inputs: &[(1, 0, 0, witness.clone()), (2, 0, 0, witness.clone())],
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
          .inscription_number,
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
        inputs: &[(3, 1, 1, witness)],
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
          .inscription_number,
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
        inputs: &[(1, 0, 0, witness.clone()), (2, 0, 0, witness.clone())],
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
          .inscription_number,
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
        inputs: &[(3, 1, 1, witness)],
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
          .inscription_number,
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
        inputs: &[(4, 1, 0, witness)],
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
          .inscription_number,
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
        inputs: &[(
          1,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });

      let first = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          1,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });

      let second = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);
      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          3,
          1,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
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

  #[test]
  fn reinscriptions_are_ordered_correctly_for_many_outpoints() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let mut inscription_ids = vec![];
      for i in 1..=21 {
        let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
          inputs: &[(
            i,
            if i == 1 { 0 } else { 1 },
            0,
            inscription("text/plain;charset=utf-8", &format!("hello {}", i)).to_witness(),
          )], // for the first inscription use coinbase, otherwise use the previous tx
          ..Default::default()
        });

        inscription_ids.push(InscriptionId { txid, index: 0 });

        context.mine_blocks(1);
      }

      let final_txid = inscription_ids.last().unwrap().txid;
      let location = SatPoint {
        outpoint: OutPoint {
          txid: final_txid,
          vout: 0,
        },
        offset: 0,
      };

      let expected_result = inscription_ids
        .iter()
        .map(|id| (location, *id))
        .collect::<Vec<(SatPoint, InscriptionId)>>();

      assert_eq!(
        expected_result,
        context
          .index
          .get_inscriptions_on_output_with_satpoints(OutPoint {
            txid: final_txid,
            vout: 0
          })
          .unwrap()
      )
    }
  }

  #[test]
  fn recover_from_reorg() {
    for mut context in Context::configurations() {
      context.index.set_durability(redb::Durability::Immediate);

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          1,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });
      let first_id = InscriptionId { txid, index: 0 };
      let first_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      context.mine_blocks(6);

      context
        .index
        .assert_inscription_location(first_id, first_location, Some(50 * COIN_VALUE));

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });
      let second_id = InscriptionId { txid, index: 0 };
      let second_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      context.mine_blocks(1);

      context
        .index
        .assert_inscription_location(second_id, second_location, Some(100 * COIN_VALUE));

      context.rpc_server.invalidate_tip();
      context.mine_blocks(2);

      context
        .index
        .assert_inscription_location(first_id, first_location, Some(50 * COIN_VALUE));

      assert!(!context.index.inscription_exists(second_id).unwrap());
    }
  }

  #[test]
  fn recover_from_3_block_deep_and_consecutive_reorg() {
    for mut context in Context::configurations() {
      context.index.set_durability(redb::Durability::Immediate);

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          1,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });
      let first_id = InscriptionId { txid, index: 0 };
      let first_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      context.mine_blocks(10);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });
      let second_id = InscriptionId { txid, index: 0 };
      let second_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      context.mine_blocks(1);

      context
        .index
        .assert_inscription_location(second_id, second_location, Some(100 * COIN_VALUE));

      context.rpc_server.invalidate_tip();
      context.rpc_server.invalidate_tip();
      context.rpc_server.invalidate_tip();

      context.mine_blocks(4);

      assert!(!context.index.inscription_exists(second_id).unwrap());

      context.rpc_server.invalidate_tip();

      context.mine_blocks(2);

      context
        .index
        .assert_inscription_location(first_id, first_location, Some(50 * COIN_VALUE));
    }
  }

  #[test]
  fn recover_from_very_unlikely_7_block_deep_reorg() {
    for mut context in Context::configurations() {
      context.index.set_durability(redb::Durability::Immediate);

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          1,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });

      context.mine_blocks(11);

      let first_id = InscriptionId { txid, index: 0 };
      let first_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hello").to_witness(),
        )],
        ..Default::default()
      });

      let second_id = InscriptionId { txid, index: 0 };
      let second_location = SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 0,
      };

      context.mine_blocks(7);

      context
        .index
        .assert_inscription_location(second_id, second_location, Some(100 * COIN_VALUE));

      for _ in 0..7 {
        context.rpc_server.invalidate_tip();
      }

      context.mine_blocks(9);

      assert!(!context.index.inscription_exists(second_id).unwrap());

      context
        .index
        .assert_inscription_location(first_id, first_location, Some(50 * COIN_VALUE));
    }
  }

  #[test]
  fn inscription_without_parent_tag_has_no_parent_entry() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert!(context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap()
        .parent
        .is_none());
    }
  }

  #[test]
  fn inscription_with_parent_tag_without_parent_has_no_parent_entry() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let parent_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let parent_inscription_id = InscriptionId {
        txid: parent_txid,
        index: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          0,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(parent_inscription_id.value()),
            ..Default::default()
          }
          .to_witness(),
        )],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert!(context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap()
        .parent
        .is_none());
    }
  }

  #[test]
  fn inscription_with_parent_tag_and_parent_has_parent_entry() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let parent_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let parent_inscription_id = InscriptionId {
        txid: parent_txid,
        index: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          1,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(parent_inscription_id.value()),
            ..Default::default()
          }
          .to_witness(),
        )],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert_eq!(
        context.index.get_parent_by_inscription_id(inscription_id),
        parent_inscription_id
      );

      assert_eq!(
        context
          .index
          .get_children_by_inscription_id(parent_inscription_id)
          .unwrap(),
        vec![inscription_id]
      );
    }
  }

  #[test]
  fn parents_can_be_in_preceding_input() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let parent_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(2);

      let parent_inscription_id = InscriptionId {
        txid: parent_txid,
        index: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[
          (2, 1, 0, Default::default()),
          (
            3,
            0,
            0,
            Inscription {
              content_type: Some("text/plain".into()),
              body: Some("hello".into()),
              parent: Some(parent_inscription_id.value()),
              ..Default::default()
            }
            .to_witness(),
          ),
        ],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert_eq!(
        context.index.get_parent_by_inscription_id(inscription_id),
        parent_inscription_id
      );

      assert_eq!(
        context
          .index
          .get_children_by_inscription_id(parent_inscription_id)
          .unwrap(),
        vec![inscription_id]
      );
    }
  }

  #[test]
  fn parents_can_be_in_following_input() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let parent_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(2);

      let parent_inscription_id = InscriptionId {
        txid: parent_txid,
        index: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[
          (
            3,
            0,
            0,
            Inscription {
              content_type: Some("text/plain".into()),
              body: Some("hello".into()),
              parent: Some(parent_inscription_id.value()),
              ..Default::default()
            }
            .to_witness(),
          ),
          (2, 1, 0, Default::default()),
        ],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert_eq!(
        context.index.get_parent_by_inscription_id(inscription_id),
        parent_inscription_id
      );

      assert_eq!(
        context
          .index
          .get_children_by_inscription_id(parent_inscription_id)
          .unwrap(),
        vec![inscription_id]
      );
    }
  }

  #[test]
  fn inscription_with_invalid_parent_tag_and_parent_has_no_parent_entry() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let parent_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let parent_inscription_id = InscriptionId {
        txid: parent_txid,
        index: 0,
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          1,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(
              parent_inscription_id
                .value()
                .into_iter()
                .chain(iter::once(0))
                .collect(),
            ),
            ..Default::default()
          }
          .to_witness(),
        )],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      assert!(context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap()
        .parent
        .is_none());
    }
  }

  #[test]
  fn inscription_with_pointer() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let inscription = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello".into()),
        pointer: Some(100u64.to_le_bytes().to_vec()),
        ..Default::default()
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription.to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 100,
        },
        Some(50 * COIN_VALUE + 100),
      );
    }
  }

  #[test]
  fn inscription_with_pointer_greater_than_output_value_assigned_default() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let inscription = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello".into()),
        pointer: Some((50 * COIN_VALUE).to_le_bytes().to_vec()),
        ..Default::default()
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription.to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

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
  fn inscription_with_pointer_into_fee_ignored_and_assigned_default_location() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let inscription = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello".into()),
        pointer: Some((25 * COIN_VALUE).to_le_bytes().to_vec()),
        ..Default::default()
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription.to_witness())],
        fee: 25 * COIN_VALUE,
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

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
  fn inscription_with_pointer_is_cursed() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let inscription = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("pointer-child".into()),
        pointer: Some(0u64.to_le_bytes().to_vec()),
        ..Default::default()
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription.to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .inscription_number,
        -1
      );
    }
  }

  #[test]
  fn inscription_with_pointer_to_parent_is_cursed_reinscription() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let parent_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("text/plain", "parent").to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let parent_inscription_id = InscriptionId {
        txid: parent_txid,
        index: 0,
      };

      let child_inscription = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("pointer-child".into()),
        parent: Some(parent_inscription_id.value()),
        pointer: Some(0u64.to_le_bytes().to_vec()),
        ..Default::default()
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0, child_inscription.to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let child_inscription_id = InscriptionId { txid, index: 0 };

      context.index.assert_inscription_location(
        parent_inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        child_inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(child_inscription_id)
          .unwrap()
          .unwrap()
          .inscription_number,
        -1
      );

      assert_eq!(
        context
          .index
          .get_parent_by_inscription_id(child_inscription_id),
        parent_inscription_id
      );

      assert_eq!(
        context
          .index
          .get_children_by_inscription_id(parent_inscription_id)
          .unwrap(),
        vec![child_inscription_id]
      );
    }
  }

  #[test]
  fn inscriptions_in_same_input_with_pointers_to_same_output() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let builder = script::Builder::new();

      let builder = Inscription {
        pointer: Some(100u64.to_le_bytes().to_vec()),
        ..Default::default()
      }
      .append_reveal_script_to_builder(builder);

      let builder = Inscription {
        pointer: Some(300_000u64.to_le_bytes().to_vec()),
        ..Default::default()
      }
      .append_reveal_script_to_builder(builder);

      let builder = Inscription {
        pointer: Some(1_000_000u64.to_le_bytes().to_vec()),
        ..Default::default()
      }
      .append_reveal_script_to_builder(builder);

      let witness = Witness::from_slice(&[builder.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        ..Default::default()
      });

      context.mine_blocks(1);

      let first = InscriptionId { txid, index: 0 };
      let second = InscriptionId { txid, index: 1 };
      let third = InscriptionId { txid, index: 2 };

      context.index.assert_inscription_location(
        first,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 100,
        },
        Some(50 * COIN_VALUE + 100),
      );

      context.index.assert_inscription_location(
        second,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 300_000,
        },
        Some(50 * COIN_VALUE + 300_000),
      );

      context.index.assert_inscription_location(
        third,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 1_000_000,
        },
        Some(50 * COIN_VALUE + 1_000_000),
      );
    }
  }

  #[test]
  fn inscriptions_in_same_input_with_pointers_to_different_outputs() {
    for context in Context::configurations() {
      context.mine_blocks_with_subsidy(1, 300_000);

      let builder = script::Builder::new();

      let builder = Inscription {
        pointer: Some(100u64.to_le_bytes().to_vec()),
        ..Default::default()
      }
      .append_reveal_script_to_builder(builder);

      let builder = Inscription {
        pointer: Some(100_111u64.to_le_bytes().to_vec()),
        ..Default::default()
      }
      .append_reveal_script_to_builder(builder);

      let builder = Inscription {
        pointer: Some(299_999u64.to_le_bytes().to_vec()),
        ..Default::default()
      }
      .append_reveal_script_to_builder(builder);

      let witness = Witness::from_slice(&[builder.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        outputs: 3,
        ..Default::default()
      });

      context.mine_blocks(1);

      let first = InscriptionId { txid, index: 0 };
      let second = InscriptionId { txid, index: 1 };
      let third = InscriptionId { txid, index: 2 };

      context.index.assert_inscription_location(
        first,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 100,
        },
        Some(50 * COIN_VALUE + 100),
      );

      context.index.assert_inscription_location(
        second,
        SatPoint {
          outpoint: OutPoint { txid, vout: 1 },
          offset: 111,
        },
        Some(50 * COIN_VALUE + 100_111),
      );

      context.index.assert_inscription_location(
        third,
        SatPoint {
          outpoint: OutPoint { txid, vout: 2 },
          offset: 99_999,
        },
        Some(50 * COIN_VALUE + 299_999),
      );
    }
  }

  #[test]
  fn inscriptions_in_different_inputs_with_pointers_to_different_outputs() {
    for context in Context::configurations() {
      context.mine_blocks(3);

      let inscription_for_second_output = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello jupiter".into()),
        pointer: Some((50 * COIN_VALUE).to_le_bytes().to_vec()),
        ..Default::default()
      };

      let inscription_for_third_output = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello mars".into()),
        pointer: Some((100 * COIN_VALUE).to_le_bytes().to_vec()),
        ..Default::default()
      };

      let inscription_for_first_output = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello world".into()),
        pointer: Some(0u64.to_le_bytes().to_vec()),
        ..Default::default()
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[
          (1, 0, 0, inscription_for_second_output.to_witness()),
          (2, 0, 0, inscription_for_third_output.to_witness()),
          (3, 0, 0, inscription_for_first_output.to_witness()),
        ],
        outputs: 3,
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_for_second_output = InscriptionId { txid, index: 0 };
      let inscription_for_third_output = InscriptionId { txid, index: 1 };
      let inscription_for_first_output = InscriptionId { txid, index: 2 };

      context.index.assert_inscription_location(
        inscription_for_second_output,
        SatPoint {
          outpoint: OutPoint { txid, vout: 1 },
          offset: 0,
        },
        Some(100 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        inscription_for_third_output,
        SatPoint {
          outpoint: OutPoint { txid, vout: 2 },
          offset: 0,
        },
        Some(150 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        inscription_for_first_output,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscriptions_in_different_inputs_with_pointers_to_same_output() {
    for context in Context::configurations() {
      context.mine_blocks(3);

      let first_inscription = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello jupiter".into()),
        ..Default::default()
      };

      let second_inscription = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello mars".into()),
        pointer: Some(1u64.to_le_bytes().to_vec()),
        ..Default::default()
      };

      let third_inscription = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello world".into()),
        pointer: Some(2u64.to_le_bytes().to_vec()),
        ..Default::default()
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[
          (1, 0, 0, first_inscription.to_witness()),
          (2, 0, 0, second_inscription.to_witness()),
          (3, 0, 0, third_inscription.to_witness()),
        ],
        outputs: 1,
        ..Default::default()
      });

      context.mine_blocks(1);

      let first_inscription_id = InscriptionId { txid, index: 0 };
      let second_inscription_id = InscriptionId { txid, index: 1 };
      let third_inscription_id = InscriptionId { txid, index: 2 };

      context.index.assert_inscription_location(
        first_inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        second_inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 1,
        },
        Some(50 * COIN_VALUE + 1),
      );

      context.index.assert_inscription_location(
        third_inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 2,
        },
        Some(50 * COIN_VALUE + 2),
      );
    }
  }

  #[test]
  fn inscriptions_with_pointers_to_same_sat_one_becomes_cursed_reinscriptions() {
    for context in Context::configurations() {
      context.mine_blocks(2);

      let inscription = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello jupiter".into()),
        ..Default::default()
      };

      let cursed_reinscription = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello mars".into()),
        pointer: Some(0u64.to_le_bytes().to_vec()),
        ..Default::default()
      };

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[
          (1, 0, 0, inscription.to_witness()),
          (2, 0, 0, cursed_reinscription.to_witness()),
        ],
        outputs: 2,
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };
      let cursed_reinscription_id = InscriptionId { txid, index: 1 };

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      context.index.assert_inscription_location(
        cursed_reinscription_id,
        SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 0,
        },
        Some(50 * COIN_VALUE),
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(inscription_id)
          .unwrap()
          .unwrap()
          .inscription_number,
        0
      );

      assert_eq!(
        context
          .index
          .get_inscription_entry(cursed_reinscription_id)
          .unwrap()
          .unwrap()
          .inscription_number,
        -1
      );
    }
  }

  #[test]
  fn inscribe_into_fee() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let inscription = Inscription::default();

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription.to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });

      let blocks = context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: blocks[0].txdata[0].txid(),
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn inscribe_into_fee_with_reduced_subsidy() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      let inscription = Inscription::default();

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription.to_witness())],
        fee: 50 * COIN_VALUE,
        ..Default::default()
      });

      let blocks = context.mine_blocks_with_subsidy(1, 25 * COIN_VALUE);

      let inscription_id = InscriptionId { txid, index: 0 };

      context.index.assert_inscription_location(
        inscription_id,
        SatPoint {
          outpoint: OutPoint {
            txid: blocks[0].txdata[0].txid(),
            vout: 0,
          },
          offset: 50 * COIN_VALUE,
        },
        Some(50 * COIN_VALUE),
      );
    }
  }

  #[test]
  fn pre_jubilee_first_reinscription_after_cursed_inscription_is_blessed() {
    for context in Context::configurations() {
      context.mine_blocks(1);

      // Before the jubilee, an inscription on a sat using a pushnum opcode is
      // cursed and not vindicated.

      let script = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([])
        .push_opcode(opcodes::all::OP_PUSHNUM_1)
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_slice(&[script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      let entry = context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap();

      assert!(Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Cursed));

      assert!(!Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Vindicated));

      let sat = entry.sat;

      assert_eq!(entry.inscription_number, -1);

      // Before the jubilee, reinscription on the same sat is not cursed and
      // not vindicated.

      let inscription = Inscription::default();

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(2, 1, 0, inscription.to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      let entry = context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap();

      assert_eq!(entry.inscription_number, 0);

      assert!(!Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Cursed));

      assert!(!Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Vindicated));

      assert_eq!(sat, entry.sat);

      // Before the jubilee, a third reinscription on the same sat is cursed
      // and not vindicated.

      let inscription = Inscription::default();

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(3, 1, 0, inscription.to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      let entry = context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap();

      assert!(Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Cursed));

      assert!(!Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Vindicated));

      assert_eq!(entry.inscription_number, -2);

      assert_eq!(sat, entry.sat);
    }
  }

  #[test]
  fn post_jubilee_first_reinscription_after_vindicated_inscription_not_vindicated() {
    for context in Context::configurations() {
      context.mine_blocks(110);
      // After the jubilee, an inscription on a sat using a pushnum opcode is
      // vindicated and not cursed.

      let script = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_slice([])
        .push_opcode(opcodes::all::OP_PUSHNUM_1)
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script();

      let witness = Witness::from_slice(&[script.into_bytes(), Vec::new()]);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, witness)],
        ..Default::default()
      });

      let inscription_id = InscriptionId { txid, index: 0 };

      context.mine_blocks(1);

      let entry = context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap();

      assert!(!Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Cursed));

      assert!(Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Vindicated));

      let sat = entry.sat;

      assert_eq!(entry.inscription_number, 0);

      // After the jubilee, a reinscription on the same is not cursed and not
      // vindicated.

      let inscription = Inscription::default();

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(111, 1, 0, inscription.to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      let entry = context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap();

      assert!(!Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Cursed));

      assert!(!Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Vindicated));

      assert_eq!(entry.inscription_number, 1);

      assert_eq!(sat, entry.sat);

      // After the jubilee, a third reinscription on the same is vindicated and
      // not cursed.

      let inscription = Inscription::default();

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(112, 1, 0, inscription.to_witness())],
        ..Default::default()
      });

      context.mine_blocks(1);

      let inscription_id = InscriptionId { txid, index: 0 };

      let entry = context
        .index
        .get_inscription_entry(inscription_id)
        .unwrap()
        .unwrap();

      assert!(!Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Cursed));

      assert!(Charm::charms(entry.charms)
        .iter()
        .any(|charm| *charm == Charm::Vindicated));

      assert_eq!(entry.inscription_number, 2);

      assert_eq!(sat, entry.sat);
    }
  }
}
