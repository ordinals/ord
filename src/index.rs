use {
  super::*,
  bitcoin::consensus::encode::serialize,
  bitcoin::BlockHeader,
  bitcoincore_rpc::{Auth, Client, RpcApi},
  rayon::iter::{IntoParallelRefIterator, ParallelIterator},
  redb::WriteStrategy,
  std::sync::atomic::{AtomicBool, Ordering},
};

mod rtx;

const HEIGHT_TO_HASH: TableDefinition<u64, [u8; 32]> = TableDefinition::new("HEIGHT_TO_HASH");
const OUTPOINT_TO_ORDINAL_RANGES: TableDefinition<[u8; 36], [u8]> =
  TableDefinition::new("OUTPOINT_TO_ORDINAL_RANGES");
const OUTPOINT_TO_TXID: TableDefinition<[u8; 36], [u8; 32]> =
  TableDefinition::new("OUTPOINT_TO_TXID");
const STATISTICS: TableDefinition<u64, u64> = TableDefinition::new("STATISTICS");

pub(crate) struct Index {
  client: Client,
  database: Database,
  database_path: PathBuf,
  height_limit: Option<u64>,
  reorged: AtomicBool,
}

#[derive(Debug, PartialEq)]
pub(crate) enum List {
  Spent(Txid),
  Unspent(Vec<(u64, u64)>),
}

#[repr(u64)]
enum Statistic {
  OutputsTraversed = 0,
}

impl From<Statistic> for u64 {
  fn from(statistic: Statistic) -> Self {
    statistic as u64
  }
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
      assert_eq!(options.chain.network(), Network::Regtest);
    }

    log::info!(
      "Connection to Bitcoin Core RPC server at {rpc_url} using credentials from `{}`",
      cookie_file.display()
    );

    let client = Client::new(&rpc_url, Auth::CookieFile(cookie_file))
      .context("Failed to connect to RPC URL")?;

    let database_path = options.data_dir()?.join("index.redb");

    let database = match unsafe { redb::Database::open(&database_path) } {
      Ok(database) => database,
      Err(redb::Error::Io(error)) if error.kind() == io::ErrorKind::NotFound => unsafe {
        Database::builder()
          .set_write_strategy(WriteStrategy::Checksum)
          .create(&database_path, options.max_index_size().0)?
      },
      Err(error) => return Err(error.into()),
    };

    let tx = database.begin_write()?;

    tx.open_table(HEIGHT_TO_HASH)?;
    tx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;
    tx.open_table(OUTPOINT_TO_TXID)?;
    tx.open_table(STATISTICS)?;

    tx.commit()?;

    Ok(Self {
      client,
      database,
      database_path,
      height_limit: options.height_limit,
      reorged: AtomicBool::new(false),
    })
  }

  pub(crate) fn print_info(&self) -> Result {
    let wtx = self.database.begin_write()?;

    let blocks_indexed = wtx
      .open_table(HEIGHT_TO_HASH)?
      .range(0..)?
      .rev()
      .next()
      .map(|(height, _hash)| height + 1)
      .unwrap_or(0);

    let utxos_indexed = wtx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?.len()?;

    let outputs_traversed = wtx
      .open_table(STATISTICS)?
      .get(&Statistic::OutputsTraversed.into())?
      .unwrap_or(0);

    let stats = wtx.stats()?;

    println!("blocks indexed: {}", blocks_indexed);
    println!("utxos indexed: {}", utxos_indexed);
    println!("outputs traversed: {}", outputs_traversed);
    println!("tree height: {}", stats.tree_height());
    println!("free pages: {}", stats.free_pages());
    println!("stored: {}", Bytes(stats.stored_bytes()));
    println!("overhead: {}", Bytes(stats.metadata_bytes()));
    println!("fragmented: {}", Bytes(stats.fragmented_bytes()));
    println!(
      "index size: {}",
      Bytes(std::fs::metadata(&self.database_path)?.len().try_into()?)
    );

    wtx.abort()?;

    Ok(())
  }

  pub(crate) fn decode_ordinal_range(bytes: [u8; 11]) -> (u64, u64) {
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

  pub(crate) fn index(&self) -> Result {
    let mut wtx = self.database.begin_write()?;

    let height = wtx
      .open_table(HEIGHT_TO_HASH)?
      .range(0..)?
      .rev()
      .next()
      .map(|(height, _hash)| height + 1)
      .unwrap_or(0);

    for (i, height) in (0..).zip(height..) {
      if let Some(height_limit) = self.height_limit {
        if height > height_limit {
          break;
        }
      }

      let done = self.index_block(&mut wtx, height)?;

      if i > 0 && i % 1000 == 0 {
        wtx.commit()?;
        wtx = self.database.begin_write()?;
      }

      if done || INTERRUPTS.load(atomic::Ordering::Relaxed) > 0 {
        break;
      }
    }

    wtx.commit()?;

    Ok(())
  }

  pub(crate) fn is_reorged(&self) -> bool {
    self.reorged.load(Ordering::Relaxed)
  }

  pub(crate) fn index_block(&self, wtx: &mut WriteTransaction, height: u64) -> Result<bool> {
    let mut height_to_hash = wtx.open_table(HEIGHT_TO_HASH)?;
    let mut outpoint_to_ordinal_ranges = wtx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;
    let mut outpoint_to_txid = wtx.open_table(OUTPOINT_TO_TXID)?;
    let mut statistics = wtx.open_table(STATISTICS)?;

    let start = Instant::now();
    let mut ordinal_ranges_written = 0;
    let mut outputs_in_block = 0;

    let mut errors = 0;
    let block = loop {
      match self.block_at_height(height) {
        Err(err) => {
          if cfg!(test) {
            return Err(err);
          }

          errors += 1;
          let seconds = 1 << errors;
          log::error!("Failed to fetch block {height}, retrying in {seconds}s: {err}");

          if seconds > 120 {
            log::error!("Would sleep for more than 120s, giving up");
            return Err(err);
          }

          thread::sleep(Duration::from_secs(seconds));
        }
        Ok(Some(block)) => break block,
        Ok(None) => {
          return Ok(true);
        }
      }
    };

    let time: DateTime<Utc> = DateTime::from_utc(
      NaiveDateTime::from_timestamp(block.header.time as i64, 0),
      Utc,
    );

    log::info!(
      "Block {height} at {} with {} transactions…",
      time,
      block.txdata.len()
    );

    if let Some(prev_height) = height.checked_sub(1) {
      let prev_hash = height_to_hash.get(&prev_height)?.unwrap();

      if prev_hash != block.header.prev_blockhash.as_ref() {
        self.reorged.store(true, Ordering::Relaxed);
        return Err(anyhow!("Reorg detected at or before {prev_height}"));
      }
    }

    let mut coinbase_inputs = VecDeque::new();

    let h = Height(height);
    if h.subsidy() > 0 {
      let start = h.starting_ordinal();
      coinbase_inputs.push_front((start.n(), (start + h.subsidy()).n()));
    }

    let txdata = block
      .txdata
      .par_iter()
      .map(|tx| (tx.txid(), tx))
      .collect::<Vec<(Txid, &Transaction)>>();

    for (tx_offset, (txid, tx)) in txdata.iter().enumerate().skip(1) {
      log::trace!("Indexing transaction {tx_offset}…");

      let mut input_ordinal_ranges = VecDeque::new();

      for input in &tx.input {
        let key = serialize(&input.previous_output).try_into().unwrap();

        let ordinal_ranges = outpoint_to_ordinal_ranges
          .get(&key)?
          .ok_or_else(|| anyhow!("Could not find outpoint {} in index", input.previous_output))?;

        for chunk in ordinal_ranges.chunks_exact(11) {
          input_ordinal_ranges.push_back(Self::decode_ordinal_range(chunk.try_into().unwrap()));
        }

        outpoint_to_ordinal_ranges.remove(&key)?;
      }

      self.index_transaction(
        *txid,
        tx,
        &mut outpoint_to_ordinal_ranges,
        &mut outpoint_to_txid,
        &mut input_ordinal_ranges,
        &mut ordinal_ranges_written,
        &mut outputs_in_block,
      )?;

      coinbase_inputs.extend(input_ordinal_ranges);
    }

    if let Some((txid, tx)) = txdata.first() {
      self.index_transaction(
        *txid,
        tx,
        &mut outpoint_to_ordinal_ranges,
        &mut outpoint_to_txid,
        &mut coinbase_inputs,
        &mut ordinal_ranges_written,
        &mut outputs_in_block,
      )?;
    }

    height_to_hash.insert(&height, &block.block_hash().as_hash().into_inner())?;

    statistics.insert(
      &Statistic::OutputsTraversed.into(),
      &(statistics
        .get(&(Statistic::OutputsTraversed.into()))?
        .unwrap_or(0)
        + outputs_in_block),
    )?;

    log::info!(
      "Wrote {ordinal_ranges_written} ordinal ranges in {}ms",
      (Instant::now() - start).as_millis(),
    );

    Ok(false)
  }

  fn begin_read(&self) -> Result<rtx::Rtx> {
    Ok(rtx::Rtx(self.database.begin_read()?))
  }

  pub(crate) fn height(&self) -> Result<Height> {
    Ok(Height(self.begin_read()?.height()?))
  }

  pub(crate) fn blocks(&self, take: u64) -> Result<Vec<(u64, BlockHash)>> {
    let mut blocks = Vec::new();

    let rtx = self.begin_read()?;

    let height = rtx.height()?;

    let height_to_hash = rtx.0.open_table(HEIGHT_TO_HASH)?;

    let mut cursor = height_to_hash
      .range(height.saturating_sub(take.saturating_sub(1))..=height)?
      .rev();

    while let Some(next) = cursor.next() {
      blocks.push((next.0, BlockHash::from_slice(next.1)?));
    }

    Ok(blocks)
  }

  fn index_transaction(
    &self,
    txid: Txid,
    tx: &Transaction,
    outpoint_to_ordinal_ranges: &mut Table<[u8; 36], [u8]>,
    #[allow(unused)] outpoint_to_txid: &mut Table<[u8; 36], [u8; 32]>,
    input_ordinal_ranges: &mut VecDeque<(u64, u64)>,
    ordinal_ranges_written: &mut u64,
    outputs_traversed: &mut u64,
  ) -> Result {
    for (vout, output) in tx.output.iter().enumerate() {
      let outpoint = OutPoint {
        vout: vout as u32,
        txid,
      };
      let mut ordinals = Vec::new();

      let mut remaining = output.value;
      while remaining > 0 {
        let range = input_ordinal_ranges
          .pop_front()
          .ok_or_else(|| anyhow!("Insufficient inputs for transaction outputs"))?;

        let count = range.1 - range.0;

        let assigned = if count > remaining {
          let middle = range.0 + remaining;
          input_ordinal_ranges.push_front((middle, range.1));
          (range.0, middle)
        } else {
          range
        };

        let base = assigned.0;
        let delta = assigned.1 - assigned.0;

        let n = base as u128 | (delta as u128) << 51;

        ordinals.extend_from_slice(&n.to_le_bytes()[0..11]);

        remaining -= assigned.1 - assigned.0;

        *ordinal_ranges_written += 1;
      }

      *outputs_traversed += 1;

      outpoint_to_ordinal_ranges.insert(&serialize(&outpoint).try_into().unwrap(), &ordinals)?;
    }

    #[cfg(any())]
    for input in &tx.input {
      outpoint_to_txid.insert(&serialize(&input.previous_output), &txid)?;
    }

    Ok(())
  }

  fn block_at_height(&self, height: u64) -> Result<Option<Block>> {
    Ok(
      self
        .client
        .get_block_hash(height)
        .into_option()?
        .map(|hash| self.client.get_block(&hash))
        .transpose()?,
    )
  }

  pub(crate) fn block_header(&self, hash: BlockHash) -> Result<Option<BlockHeader>> {
    self.client.get_block_header(&hash).into_option()
  }

  pub(crate) fn block_with_hash(&self, hash: BlockHash) -> Result<Option<Block>> {
    self.client.get_block(&hash).into_option()
  }

  pub(crate) fn transaction(&self, txid: Txid) -> Result<Option<Transaction>> {
    self.client.get_raw_transaction(&txid, None).into_option()
  }

  pub(crate) fn find(&self, ordinal: u64) -> Result<Option<SatPoint>> {
    if self.height()? < Ordinal(ordinal).height() {
      return Ok(None);
    }

    let rtx = self.database.begin_read()?;

    let outpoint_to_ordinal_ranges = rtx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;

    let mut cursor = outpoint_to_ordinal_ranges.range([0; 36]..)?;

    while let Some((key, value)) = cursor.next() {
      let mut offset = 0;
      for chunk in value.chunks_exact(11) {
        let (start, end) = Index::decode_ordinal_range(chunk.try_into().unwrap());
        if start <= ordinal && ordinal < end {
          let outpoint: OutPoint = Decodable::consensus_decode(key.as_slice())?;
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

  pub(crate) fn list_inner(&self, outpoint: &[u8]) -> Result<Option<Vec<u8>>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(OUTPOINT_TO_ORDINAL_RANGES)?
        .get(outpoint.try_into().unwrap())?
        .map(|outpoint| outpoint.to_vec()),
    )
  }

  pub(crate) fn list(&self, outpoint: OutPoint) -> Result<Option<List>> {
    let outpoint_encoded = serialize(&outpoint);

    let ordinal_ranges = self.list_inner(&outpoint_encoded)?;

    match ordinal_ranges {
      Some(ordinal_ranges) => Ok(Some(List::Unspent(
        ordinal_ranges
          .chunks_exact(11)
          .map(|chunk| Self::decode_ordinal_range(chunk.try_into().unwrap()))
          .collect(),
      ))),
      None => Ok(
        self
          .database
          .begin_read()?
          .open_table(OUTPOINT_TO_TXID)?
          .get(&outpoint_encoded.try_into().unwrap())?
          .map(|txid| Txid::consensus_decode(txid.as_slice()))
          .transpose()?
          .map(List::Spent),
      ),
    }
  }

  pub(crate) fn blocktime(&self, height: Height) -> Result<Blocktime> {
    let height = height.n();

    match self.block_at_height(height)? {
      Some(block) => Ok(Blocktime::Confirmed(block.header.time.into())),
      None => {
        let tx = self.database.begin_read()?;

        let current = tx
          .open_table(HEIGHT_TO_HASH)?
          .range(0..)?
          .rev()
          .next()
          .map(|(height, _hash)| height)
          .unwrap_or(0);

        let expected_blocks = height.checked_sub(current).with_context(|| {
          format!("Current {current} height is greater than ordinal height {height}")
        })?;

        Ok(Blocktime::Expected(
          Utc::now().timestamp() + 10 * 60 * expected_blocks as i64,
        ))
      }
    }
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
    fn new() -> Self {
      Self::with_args("")
    }

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
      index.index().unwrap();

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
      context.index.index().unwrap();
      assert_eq!(context.index.height().unwrap(), 0);
    }

    {
      let context = Context::with_args("--height-limit 1");
      context.rpc_server.mine_blocks(1);
      context.index.index().unwrap();
      assert_eq!(context.index.height().unwrap(), 1);
    }
  }

  #[test]
  fn list_first_coinbase_transaction() {
    let context = Context::new();
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
    let context = Context::new();
    let txid = context.rpc_server.mine_blocks(1)[0].txdata[0].txid();
    context.index.index().unwrap();
    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![(50 * COIN_VALUE, 100 * COIN_VALUE)])
    )
  }

  #[test]
  fn list_split_ranges_are_tracked_correctly() {
    let context = Context::new();

    context.rpc_server.mine_blocks(1);
    let split_coinbase_output = test_bitcoincore_rpc::TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 2,
      fee: 0,
    };
    let txid = context.rpc_server.broadcast_tx(split_coinbase_output);

    context.rpc_server.mine_blocks(1);
    context.index.index().unwrap();

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
    let context = Context::new();

    context.rpc_server.mine_blocks(2);
    let merge_coinbase_outputs = test_bitcoincore_rpc::TransactionTemplate {
      input_slots: &[(1, 0, 0), (2, 0, 0)],
      output_count: 1,
      fee: 0,
    };

    let txid = context.rpc_server.broadcast_tx(merge_coinbase_outputs);
    context.rpc_server.mine_blocks(1);
    context.index.index().unwrap();

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
    let context = Context::new();

    context.rpc_server.mine_blocks(1);
    let fee_paying_tx = test_bitcoincore_rpc::TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 2,
      fee: 10,
    };
    let txid = context.rpc_server.broadcast_tx(fee_paying_tx);
    let coinbase_txid = context.rpc_server.mine_blocks(1)[0].txdata[0].txid();
    context.index.index().unwrap();

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
    let context = Context::new();

    context.rpc_server.mine_blocks(2);
    let first_fee_paying_tx = test_bitcoincore_rpc::TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 10,
    };
    let second_fee_paying_tx = test_bitcoincore_rpc::TransactionTemplate {
      input_slots: &[(2, 0, 0)],
      output_count: 1,
      fee: 10,
    };
    context.rpc_server.broadcast_tx(first_fee_paying_tx);
    context.rpc_server.broadcast_tx(second_fee_paying_tx);

    let coinbase_txid = context.rpc_server.mine_blocks(1)[0].txdata[0].txid();
    context.index.index().unwrap();

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
    let context = Context::new();

    context.rpc_server.mine_blocks(1);
    let no_value_output = test_bitcoincore_rpc::TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 50 * COIN_VALUE,
    };
    let txid = context.rpc_server.broadcast_tx(no_value_output);
    context.rpc_server.mine_blocks(1);
    context.index.index().unwrap();

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![])
    );
  }

  #[test]
  fn list_null_input() {
    let context = Context::new();

    context.rpc_server.mine_blocks(1);
    let no_value_output = test_bitcoincore_rpc::TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 50 * COIN_VALUE,
    };
    context.rpc_server.broadcast_tx(no_value_output);
    context.rpc_server.mine_blocks(1);

    let no_value_input = test_bitcoincore_rpc::TransactionTemplate {
      input_slots: &[(2, 1, 0)],
      output_count: 1,
      fee: 0,
    };
    let txid = context.rpc_server.broadcast_tx(no_value_input);
    context.rpc_server.mine_blocks(1);
    context.index.index().unwrap();

    assert_eq!(
      context.index.list(OutPoint::new(txid, 0)).unwrap().unwrap(),
      List::Unspent(vec![])
    );
  }

  #[test]
  fn list_unknown_output() {
    let context = Context::new();

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
    let context = Context::new();
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
    let context = Context::new();
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
    let context = Context::new();
    context.rpc_server.mine_blocks(1);
    context.index.index().unwrap();
    assert_eq!(
      context.index.find(50 * COIN_VALUE).unwrap().unwrap(),
      SatPoint {
        outpoint: "9068a11b8769174363376b606af9a4b8b29dd7b13d013f4b0cbbd457db3c3ce5:0"
          .parse()
          .unwrap(),
        offset: 0,
      }
    )
  }

  #[test]
  fn find_unmined_ordinal() {
    let context = Context::new();
    assert_eq!(context.index.find(50 * COIN_VALUE).unwrap(), None);
  }
}
