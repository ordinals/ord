use {
  self::updater::Updater,
  super::*,
  bitcoin::BlockHeader,
  bitcoincore_rpc::{json::GetBlockHeaderResult, Auth, Client},
  indicatif::{ProgressBar, ProgressStyle},
  log::log_enabled,
  redb::{Database, ReadableTable, Table, TableDefinition, WriteStrategy, WriteTransaction},
  std::collections::HashMap,
  std::sync::atomic::{AtomicBool, Ordering},
};

mod rtx;
mod updater;

type BlockHashArray = [u8; 32];
type InscriptionIdArray = [u8; 32];
type OrdinalRangeArray = [u8; 11];
type OutPointArray = [u8; 36];
type SatPointArray = [u8; 44];

const HEIGHT_TO_BLOCK_HASH: TableDefinition<u64, &BlockHashArray> =
  TableDefinition::new("HEIGHT_TO_BLOCK_HASH");
const ORDINAL_TO_INSCRIPTION_ID: TableDefinition<u64, &InscriptionIdArray> =
  TableDefinition::new("ORDINAL_TO_INSCRIPTION_ID");
const ORDINAL_TO_SATPOINT: TableDefinition<u64, &SatPointArray> =
  TableDefinition::new("ORDINAL_TO_SATPOINT");
const OUTPOINT_TO_ORDINAL_RANGES: TableDefinition<&OutPointArray, [u8]> =
  TableDefinition::new("OUTPOINT_TO_ORDINAL_RANGES");
const STATISTIC_TO_COUNT: TableDefinition<u64, u64> = TableDefinition::new("STATISTIC_TO_COUNT");
const INSCRIPTION_ID_TO_INSCRIPTION: TableDefinition<&InscriptionIdArray, str> =
  TableDefinition::new("INSCRIPTION_ID_TO_INSCRIPTION");
const WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP: TableDefinition<u64, u128> =
  TableDefinition::new("WRITE_TRANSACTION_START_BLOCK_COUNT_TO_TIMESTAMP");
const INSCRIPTION_ID_TO_SATPOINT: TableDefinition<&InscriptionIdArray, &SatPointArray> =
  TableDefinition::new("INSCRIPTION_ID_TO_SATPOINT");
const SATPOINT_TO_INSCRIPTION_ID: TableDefinition<&SatPointArray, &InscriptionIdArray> =
  TableDefinition::new("SATPOINT_TO_INSCRIPTION_ID");

fn encode_outpoint(outpoint: OutPoint) -> OutPointArray {
  let mut array = [0; 36];
  outpoint
    .consensus_encode(&mut array.as_mut_slice())
    .unwrap();
  array
}

fn encode_satpoint(satpoint: SatPoint) -> SatPointArray {
  let mut array = [0; 44];
  satpoint
    .consensus_encode(&mut array.as_mut_slice())
    .unwrap();
  array
}

fn decode_satpoint(array: SatPointArray) -> SatPoint {
  Decodable::consensus_decode(&mut io::Cursor::new(array)).unwrap()
}

fn decode_outpoint(array: OutPointArray) -> OutPoint {
  Decodable::consensus_decode(&mut io::Cursor::new(array)).unwrap()
}

pub(crate) struct Index {
  auth: Auth,
  chain: Chain,
  client: Client,
  database: Database,
  database_path: PathBuf,
  genesis_block_coinbase_transaction: Transaction,
  genesis_block_coinbase_txid: Txid,
  height_limit: Option<u64>,
  reorged: AtomicBool,
  rpc_url: String,
}

#[derive(Debug, PartialEq)]
pub(crate) enum List {
  Spent,
  Unspent(Vec<(u64, u64)>),
}

#[derive(Copy, Clone)]
#[repr(u64)]
pub(crate) enum Statistic {
  OutputsTraversed = 0,
  Commits = 1,
  OrdinalRanges = 2,
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
  pub(crate) branch_pages: usize,
  pub(crate) fragmented_bytes: usize,
  pub(crate) index_file_size: u64,
  pub(crate) leaf_pages: usize,
  pub(crate) metadata_bytes: usize,
  pub(crate) ordinal_ranges: u64,
  pub(crate) outputs_traversed: u64,
  pub(crate) page_size: usize,
  pub(crate) stored_bytes: usize,
  pub(crate) transactions: Vec<TransactionInfo>,
  pub(crate) tree_height: usize,
  pub(crate) utxos_indexed: usize,
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
    let rpc_url = options.rpc_url();
    let cookie_file = options.cookie_file()?;

    if cfg!(test) {
      // The default max database size is 10 MiB for Regtest and 1 TiB
      // for all other networks. A larger database takes longer to
      // initialize, so unit tests should use the regtest network.
      assert_eq!(options.chain, Chain::Regtest);
    }

    log::info!(
      "Connecting to Bitcoin Core RPC server at {rpc_url} using credentials from `{}`",
      cookie_file.display()
    );

    let auth = Auth::CookieFile(cookie_file);

    let client = Client::new(&rpc_url, auth.clone()).context("failed to connect to RPC URL")?;

    let data_dir = options.data_dir()?;

    if let Err(err) = fs::create_dir_all(&data_dir) {
      bail!("failed to create data dir `{}`: {err}", data_dir.display());
    }

    let database_path = if let Some(database_path) = &options.index {
      database_path.clone()
    } else {
      data_dir.join("index.redb")
    };

    let database = match unsafe { redb::Database::open(&database_path) } {
      Ok(database) => database,
      Err(redb::Error::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
        let database = unsafe {
          Database::builder()
            .set_write_strategy(if cfg!(test) {
              WriteStrategy::Checksum
            } else {
              WriteStrategy::TwoPhase
            })
            .create(&database_path)?
        };
        let tx = database.begin_write()?;

        #[cfg(test)]
        let tx = {
          let mut tx = tx;
          tx.set_durability(redb::Durability::None);
          tx
        };

        tx.open_table(HEIGHT_TO_BLOCK_HASH)?;
        tx.open_table(INSCRIPTION_ID_TO_INSCRIPTION)?;
        tx.open_table(INSCRIPTION_ID_TO_SATPOINT)?;
        tx.open_table(ORDINAL_TO_INSCRIPTION_ID)?;
        tx.open_table(ORDINAL_TO_SATPOINT)?;
        tx.open_table(SATPOINT_TO_INSCRIPTION_ID)?;
        tx.open_table(STATISTIC_TO_COUNT)?;
        tx.open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?;

        if options.index_ordinals {
          tx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;
        }

        tx.commit()?;

        database
      }
      Err(error) => return Err(error.into()),
    };

    let genesis_block_coinbase_transaction =
      options.chain.genesis_block().coinbase().unwrap().clone();

    Ok(Self {
      genesis_block_coinbase_txid: genesis_block_coinbase_transaction.txid(),
      auth,
      chain: options.chain,
      client,
      database,
      database_path,
      genesis_block_coinbase_transaction,
      height_limit: options.height_limit,
      reorged: AtomicBool::new(false),
      rpc_url,
    })
  }

  pub(crate) fn has_ordinal_index(&self) -> Result<bool> {
    match self.begin_read()?.0.open_table(OUTPOINT_TO_ORDINAL_RANGES) {
      Ok(_) => Ok(true),
      Err(redb::Error::TableDoesNotExist(_)) => Ok(false),
      Err(err) => Err(err.into()),
    }
  }

  fn require_ordinal_index(&self, feature: &str) -> Result {
    if !self.has_ordinal_index()? {
      bail!("{feature} requires `--index-ordinals` flag")
    }

    Ok(())
  }

  pub(crate) fn info(&self) -> Result<Info> {
    let wtx = self.begin_write()?;

    let stats = wtx.stats()?;

    let info = {
      let statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;
      Info {
        blocks_indexed: wtx
          .open_table(HEIGHT_TO_BLOCK_HASH)?
          .range(0..)?
          .rev()
          .next()
          .map(|(height, _hash)| height + 1)
          .unwrap_or(0),
        branch_pages: stats.branch_pages(),
        fragmented_bytes: stats.fragmented_bytes(),
        index_file_size: fs::metadata(&self.database_path)?.len(),
        leaf_pages: stats.leaf_pages(),
        metadata_bytes: stats.metadata_bytes(),
        ordinal_ranges: statistic_to_count
          .get(&Statistic::OrdinalRanges.key())?
          .unwrap_or(0),
        outputs_traversed: statistic_to_count
          .get(&Statistic::OutputsTraversed.key())?
          .unwrap_or(0),
        page_size: stats.page_size(),
        stored_bytes: stats.stored_bytes(),
        transactions: wtx
          .open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?
          .range(0..)?
          .map(
            |(starting_block_count, starting_timestamp)| TransactionInfo {
              starting_block_count,
              starting_timestamp,
            },
          )
          .collect(),
        tree_height: stats.tree_height(),
        utxos_indexed: wtx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?.len()?,
      }
    };

    Ok(info)
  }

  pub(crate) fn decode_ordinal_range(bytes: OrdinalRangeArray) -> (u64, u64) {
    let n = u128::from_le_bytes([
      bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
      bytes[9], bytes[10], 0, 0, 0, 0, 0,
    ]);

    // 51 bit base
    let base = (n & ((1 << 51) - 1)) as u64;
    // 33 bit delta
    let delta = (n >> 51) as u64;

    (base, base + delta)
  }

  pub(crate) fn update(&self) -> Result {
    Updater::update(self)
  }

  pub(crate) fn is_reorged(&self) -> bool {
    self.reorged.load(Ordering::Relaxed)
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
    statistic_to_count.insert(
      &statistic.key(),
      &(statistic_to_count.get(&(statistic.key()))?.unwrap_or(0) + n),
    )?;
    Ok(())
  }

  #[cfg(test)]
  pub(crate) fn statistic(&self, statistic: Statistic) -> Result<u64> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(STATISTIC_TO_COUNT)?
        .get(&statistic.key())?
        .unwrap_or(0),
    )
  }

  pub(crate) fn height(&self) -> Result<Option<Height>> {
    self.begin_read()?.height()
  }

  pub(crate) fn block_count(&self) -> Result<u64> {
    self.begin_read()?.block_count()
  }

  pub(crate) fn blocks(&self, take: usize) -> Result<Vec<(u64, BlockHash)>> {
    let mut blocks = Vec::new();

    let rtx = self.begin_read()?;

    let block_count = rtx.block_count()?;

    let height_to_block_hash = rtx.0.open_table(HEIGHT_TO_BLOCK_HASH)?;

    for next in height_to_block_hash.range(0..block_count)?.rev().take(take) {
      blocks.push((next.0, BlockHash::from_slice(next.1)?));
    }

    Ok(blocks)
  }

  pub(crate) fn rare_ordinal_satpoints(&self) -> Result<Vec<(Ordinal, SatPoint)>> {
    self.require_ordinal_index("looking up rare ordinals")?;

    let mut result = Vec::new();

    let rtx = self.database.begin_read()?;

    let ordinal_to_satpoint = rtx.open_table(ORDINAL_TO_SATPOINT)?;

    for (ordinal, satpoint) in ordinal_to_satpoint.range(0..)? {
      result.push((Ordinal(ordinal), decode_satpoint(*satpoint)));
    }

    Ok(result)
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

  pub(crate) fn get_inscription_by_ordinal(&self, ordinal: Ordinal) -> Result<Option<Inscription>> {
    let db = self.database.begin_read()?;
    let table = db.open_table(ORDINAL_TO_INSCRIPTION_ID)?;

    let Some(txid) = table.get(&ordinal.n())? else {
      return Ok(None);
    };

    Ok(
      self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_ID_TO_INSCRIPTION)?
        .get(txid)?
        .map(|inscription| {
          serde_json::from_str(inscription)
            .expect("failed to deserialize inscription (JSON) from database")
        }),
    )
  }

  pub(crate) fn get_inscription_by_inscription_id(
    &self,
    txid: Txid,
  ) -> Result<Option<(Inscription, SatPoint)>> {
    let inscription = self
      .database
      .begin_read()?
      .open_table(INSCRIPTION_ID_TO_INSCRIPTION)?
      .get(txid.as_inner())?
      .map(|inscription| {
        serde_json::from_str(inscription)
          .expect("failed to deserialize inscription (JSON) from database")
      });

    let inscription = match inscription {
      Some(inscription) => inscription,
      None => return Ok(None),
    };

    let satpoint = decode_satpoint(
      *self
        .database
        .begin_read()?
        .open_table(INSCRIPTION_ID_TO_SATPOINT)?
        .get(txid.as_inner())?
        .ok_or_else(|| anyhow!("no satpoint for inscription"))?,
    );

    Ok(Some((inscription, satpoint)))
  }

  pub(crate) fn transaction(&self, txid: Txid) -> Result<Option<Transaction>> {
    if txid == self.genesis_block_coinbase_txid {
      Ok(Some(self.genesis_block_coinbase_transaction.clone()))
    } else {
      self.client.get_raw_transaction(&txid, None).into_option()
    }
  }

  pub(crate) fn is_transaction_in_active_chain(&self, txid: Txid) -> Result<bool> {
    Ok(
      self
        .client
        .get_raw_transaction_info(&txid, None)
        .into_option()?
        .and_then(|transaction_info| {
          transaction_info
            .confirmations
            .map(|confirmations| confirmations > 0)
        })
        .unwrap_or(false),
    )
  }

  pub(crate) fn find(&self, ordinal: u64) -> Result<Option<SatPoint>> {
    self.require_ordinal_index("find")?;

    let rtx = self.begin_read()?;

    if rtx.block_count()? <= Ordinal(ordinal).height().n() {
      return Ok(None);
    }

    let outpoint_to_ordinal_ranges = rtx.0.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;

    for (key, value) in outpoint_to_ordinal_ranges.range([0; 36]..)? {
      let mut offset = 0;
      for chunk in value.chunks_exact(11) {
        let (start, end) = Index::decode_ordinal_range(chunk.try_into().unwrap());
        if start <= ordinal && ordinal < end {
          let outpoint = decode_outpoint(*key);
          return Ok(Some(SatPoint {
            outpoint,
            offset: offset + ordinal - start,
          }));
        }
        offset += end - start;
      }
    }

    Ok(None)
  }

  fn list_inner(&self, outpoint: OutPointArray) -> Result<Option<Vec<u8>>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(OUTPOINT_TO_ORDINAL_RANGES)?
        .get(&outpoint)?
        .map(|outpoint| outpoint.to_vec()),
    )
  }

  pub(crate) fn list(&self, outpoint: OutPoint) -> Result<Option<List>> {
    self.require_ordinal_index("list")?;

    let outpoint_encoded = encode_outpoint(outpoint);

    let ordinal_ranges = self.list_inner(outpoint_encoded)?;

    match ordinal_ranges {
      Some(ordinal_ranges) => Ok(Some(List::Unspent(
        ordinal_ranges
          .chunks_exact(11)
          .map(|chunk| Self::decode_ordinal_range(chunk.try_into().unwrap()))
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

  pub(crate) fn blocktime(&self, height: Height) -> Result<Blocktime> {
    let height = height.n();

    match self.get_block_by_height(height)? {
      Some(block) => Ok(Blocktime::Confirmed(block.header.time.into())),
      None => {
        let tx = self.database.begin_read()?;

        let current = tx
          .open_table(HEIGHT_TO_BLOCK_HASH)?
          .range(0..)?
          .rev()
          .next()
          .map(|(height, _hash)| height)
          .unwrap_or(0);

        let expected_blocks = height.checked_sub(current).with_context(|| {
          format!("current {current} height is greater than ordinal height {height}")
        })?;

        Ok(Blocktime::Expected(
          Utc::now().timestamp() + 10 * 60 * expected_blocks as i64,
        ))
      }
    }
  }

  pub(crate) fn get_inscription_satpoints(&self) -> Result<Vec<SatPoint>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(SATPOINT_TO_INSCRIPTION_ID)?
        .range([0; 44]..)?
        .map(|(satpoint, _id)| decode_satpoint(*satpoint))
        .collect(),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct Context {
    rpc_server: test_bitcoincore_rpc::Handle,
    #[allow(unused)]
    tempdir: TempDir,
    index: Index,
  }

  impl Context {
    fn with_args(args: &str) -> Self {
      let rpc_server = test_bitcoincore_rpc::spawn();

      let tempdir = TempDir::new().unwrap();
      let cookie_file = tempdir.path().join("cookie");
      fs::write(&cookie_file, "username:password").unwrap();
      let options = Options::try_parse_from(
        format!(
          "
          ord
          --rpc-url {}
          --data-dir {}
          --cookie-file {}
          --chain regtest
          {args}
        ",
          rpc_server.url(),
          tempdir.path().display(),
          cookie_file.display(),
        )
        .split_whitespace(),
      )
      .unwrap();
      let index = Index::open(&options).unwrap();
      index.update().unwrap();

      Self {
        rpc_server,
        tempdir,
        index,
      }
    }
  }

  #[test]
  fn height_limit() {
    {
      let context = Context::with_args("--height-limit 0");
      context.rpc_server.mine_blocks(1);
      context.index.update().unwrap();
      assert_eq!(context.index.height().unwrap(), None);
      assert_eq!(context.index.block_count().unwrap(), 0);
    }

    {
      let context = Context::with_args("--height-limit 1");
      context.rpc_server.mine_blocks(1);
      context.index.update().unwrap();
      assert_eq!(context.index.height().unwrap(), Some(Height(0)));
      assert_eq!(context.index.block_count().unwrap(), 1);
    }

    {
      let context = Context::with_args("--height-limit 2");
      context.rpc_server.mine_blocks(2);
      context.index.update().unwrap();
      assert_eq!(context.index.height().unwrap(), Some(Height(1)));
      assert_eq!(context.index.block_count().unwrap(), 2);
    }
  }

  #[test]
  fn list_first_coinbase_transaction() {
    let context = Context::with_args("--index-ordinals");
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
    let context = Context::with_args("--index-ordinals");
    let txid = context.rpc_server.mine_blocks(1)[0].txdata[0].txid();
    context.index.update().unwrap();
    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![(50 * COIN_VALUE, 100 * COIN_VALUE)])
    )
  }

  #[test]
  fn list_split_ranges_are_tracked_correctly() {
    let context = Context::with_args("--index-ordinals");

    context.rpc_server.mine_blocks(1);
    let split_coinbase_output = TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 2,
      fee: 0,
    };
    let txid = context.rpc_server.broadcast_tx(split_coinbase_output);

    context.rpc_server.mine_blocks(1);
    context.index.update().unwrap();

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
    let context = Context::with_args("--index-ordinals");

    context.rpc_server.mine_blocks(2);
    let merge_coinbase_outputs = TransactionTemplate {
      input_slots: &[(1, 0, 0), (2, 0, 0)],
      output_count: 1,
      fee: 0,
    };

    let txid = context.rpc_server.broadcast_tx(merge_coinbase_outputs);
    context.rpc_server.mine_blocks(1);
    context.index.update().unwrap();

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
    let context = Context::with_args("--index-ordinals");

    context.rpc_server.mine_blocks(1);
    let fee_paying_tx = TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 2,
      fee: 10,
    };
    let txid = context.rpc_server.broadcast_tx(fee_paying_tx);
    let coinbase_txid = context.rpc_server.mine_blocks(1)[0].txdata[0].txid();
    context.index.update().unwrap();

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
    let context = Context::with_args("--index-ordinals");

    context.rpc_server.mine_blocks(2);
    let first_fee_paying_tx = TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 10,
    };
    let second_fee_paying_tx = TransactionTemplate {
      input_slots: &[(2, 0, 0)],
      output_count: 1,
      fee: 10,
    };
    context.rpc_server.broadcast_tx(first_fee_paying_tx);
    context.rpc_server.broadcast_tx(second_fee_paying_tx);

    let coinbase_txid = context.rpc_server.mine_blocks(1)[0].txdata[0].txid();
    context.index.update().unwrap();

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
    let context = Context::with_args("--index-ordinals");

    context.rpc_server.mine_blocks(1);
    let no_value_output = TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 50 * COIN_VALUE,
    };
    let txid = context.rpc_server.broadcast_tx(no_value_output);
    context.rpc_server.mine_blocks(1);
    context.index.update().unwrap();

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![])
    );
  }

  #[test]
  fn list_null_input() {
    let context = Context::with_args("--index-ordinals");

    context.rpc_server.mine_blocks(1);
    let no_value_output = TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 50 * COIN_VALUE,
    };
    context.rpc_server.broadcast_tx(no_value_output);
    context.rpc_server.mine_blocks(1);

    let no_value_input = TransactionTemplate {
      input_slots: &[(2, 1, 0)],
      output_count: 1,
      fee: 0,
    };
    let txid = context.rpc_server.broadcast_tx(no_value_input);
    context.rpc_server.mine_blocks(1);
    context.index.update().unwrap();

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![])
    );
  }

  #[test]
  fn list_spent_output() {
    let context = Context::with_args("--index-ordinals");
    context.rpc_server.mine_blocks(1);
    context.rpc_server.broadcast_tx(TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 0,
    });
    context.rpc_server.mine_blocks(1);
    context.index.update().unwrap();
    let txid = context.rpc_server.tx(1, 0).txid();
    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Spent,
    );
  }

  #[test]
  fn list_unknown_output() {
    let context = Context::with_args("--index-ordinals");

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
  fn find_first_ordinal() {
    let context = Context::with_args("--index-ordinals");
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
  fn find_second_ordinal() {
    let context = Context::with_args("--index-ordinals");
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
  fn find_first_ordinal_of_second_block() {
    let context = Context::with_args("--index-ordinals");
    context.rpc_server.mine_blocks(1);
    context.index.update().unwrap();
    assert_eq!(
      context.index.find(50 * COIN_VALUE).unwrap().unwrap(),
      SatPoint {
        outpoint: "0c4eb1fa83a7d6ce0e21e5e616a96e83a7b1658170fb544acf6f5c6a2d4b3f90:0"
          .parse()
          .unwrap(),
        offset: 0,
      }
    )
  }

  #[test]
  fn find_unmined_ordinal() {
    let context = Context::with_args("--index-ordinals");
    assert_eq!(context.index.find(50 * COIN_VALUE).unwrap(), None);
  }

  #[test]
  fn find_first_satoshi_spent_in_second_block() {
    let context = Context::with_args("--index-ordinals");
    context.rpc_server.mine_blocks(1);
    let spend_txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 0,
    });
    context.rpc_server.mine_blocks(1);
    context.index.update().unwrap();
    assert_eq!(
      context.index.find(50 * COIN_VALUE).unwrap().unwrap(),
      SatPoint {
        outpoint: OutPoint::new(spend_txid, 0),
        offset: 0,
      }
    )
  }
}
