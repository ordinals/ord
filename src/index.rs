use super::*;

const BLOCK_OFFSETS: &str = "block_offsets";
const CHILDREN: &str = "children";
const HEIGHTS: &str = "heights";
const HEIGHTS_TO_HASHES: &str = "height_to_hashes";
const UTXORDS: &str = "utxords";

pub(crate) struct Index {
  blocksdir: PathBuf,
  database: Database,
}

impl Index {
  pub(crate) fn new(blocksdir: Option<&Path>) -> Result<Self> {
    let blocksdir = if let Some(blocksdir) = blocksdir {
      blocksdir.to_owned()
    } else if cfg!(target_os = "macos") {
      dirs::home_dir()
        .ok_or("Unable to retrieve home directory")?
        .join("Library/Application Support/Bitcoin/blocks")
    } else if cfg!(target_os = "windows") {
      dirs::data_dir()
        .ok_or("Unable to retrieve home directory")?
        .join("Bitcoin/blocks")
    } else {
      dirs::home_dir()
        .ok_or("Unable to retrieve home directory")?
        .join(".bitcoin/blocks")
    };

    let index = Self {
      database: unsafe { Database::open("index.redb", 4096 * 1024 * 1024 * 10)? },
      blocksdir,
    };

    index.index_blockfile()?;

    index.index_ranges()?;

    Ok(index)
  }

  fn index_ranges(&self) -> Result {
    let mut height = 0;
    while let Some(block) = self.block(height)? {
      eprint!(".");
      let wtx = self.database.begin_write()?;
      let mut utxords: Table<[u8], [u8]> = wtx.open_table(UTXORDS)?;

      let mut coinbase_inputs = VecDeque::new();

      let h = Height(height);
      if let Some(start) = h.starting_ordinal() {
        coinbase_inputs.push_front((start.n(), (start + h.subsidy()).n()));
      }

      for tx in &block.txdata[1..] {
        let mut input_ordinal_ranges = VecDeque::new();
        for input in &tx.input {
          let mut key = Vec::new();
          input.previous_output.consensus_encode(&mut key)?;
          let ordinal_ranges = utxords.get(key.as_slice())?.unwrap();

          for chunk in ordinal_ranges.to_value().chunks_exact(16) {
            let start = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
            let end = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
            input_ordinal_ranges.push_back((start, end));
          }
        }
        for (vout, output) in tx.output.iter().enumerate() {
          let mut ordinals = Vec::new();
          let mut remaining = output.value;
          while remaining > 0 {
            let range = input_ordinal_ranges.pop_front().unwrap();
            let count = range.1 - range.0;
            let assigned = if count > remaining {
              let split = range.0 + remaining;
              input_ordinal_ranges.push_front((split, range.1));
              (range.0, split)
            } else {
              range
            };
            ordinals.extend_from_slice(&assigned.0.to_le_bytes());
            ordinals.extend_from_slice(&assigned.1.to_le_bytes());
            remaining -= assigned.1 - assigned.0;
          }
          let outpoint = OutPoint {
            txid: tx.txid(),
            vout: vout as u32,
          };
          let mut key = Vec::new();
          outpoint.consensus_encode(&mut key)?;
          utxords.insert(&key, &ordinals)?;
        }
        coinbase_inputs.extend(&input_ordinal_ranges);
      }

      {
        let tx = &block.txdata[0];
        for (vout, output) in tx.output.iter().enumerate() {
          let mut ordinals = Vec::new();
          let mut remaining = output.value;
          while remaining > 0 {
            let range = coinbase_inputs.pop_front().unwrap();
            let count = range.1 - range.0;
            let assigned = if count > remaining {
              let split = range.0 + remaining;
              coinbase_inputs.push_front((split, range.1));
              (range.0, split)
            } else {
              range
            };
            ordinals.extend_from_slice(&assigned.0.to_le_bytes());
            ordinals.extend_from_slice(&assigned.1.to_le_bytes());
            remaining -= assigned.1 - assigned.0;
          }
          let outpoint = OutPoint {
            txid: tx.txid(),
            vout: vout as u32,
          };
          let mut key = Vec::new();
          outpoint.consensus_encode(&mut key)?;
          utxords.insert(&key, &ordinals)?;
        }
      }

      wtx.commit()?;
      height += 1;
    }

    Ok(())
  }

  fn index_blockfile(&self) -> Result {
    {
      let tx = self.database.begin_write()?;

      let mut children: MultimapTable<[u8], [u8]> = tx.open_multimap_table(CHILDREN)?;

      let mut block_offsets: Table<[u8], u64> = tx.open_table(BLOCK_OFFSETS)?;

      let blocks = fs::read(self.blocksdir.join("blk00000.dat"))?;

      let mut offset = 0;

      let mut count = 0;

      loop {
        eprint!(".");
        if offset == blocks.len() {
          break;
        }

        let range = Self::block_range_at(&blocks, offset)?;

        let block = Block::consensus_decode(&blocks[range.clone()])?;

        children.insert(&block.header.prev_blockhash, &block.block_hash())?;

        block_offsets.insert(&block.block_hash(), &(offset as u64))?;

        offset = range.end;

        count += 1;
      }

      log::info!("Inserted {} blocks…", count);

      tx.commit()?;
    }

    {
      let write = self.database.begin_write()?;

      let mut heights: Table<[u8], u64> = write.open_table(HEIGHTS)?;
      let mut heights_to_hashes: Table<u64, [u8]> = write.open_table(HEIGHTS_TO_HASHES)?;

      heights.insert(genesis_block(Network::Bitcoin).block_hash().deref(), &0)?;
      heights_to_hashes.insert(&0, genesis_block(Network::Bitcoin).block_hash().deref())?;

      let read = self.database.begin_read()?;

      let children: ReadOnlyMultimapTable<[u8], [u8]> = read.open_multimap_table(CHILDREN)?;

      let mut queue = vec![(
        genesis_block(Network::Bitcoin)
          .block_hash()
          .deref()
          .to_vec(),
        0,
      )];

      while let Some((block, height)) = queue.pop() {
        heights.insert(block.as_ref(), &height)?;
        heights_to_hashes.insert(&height, block.as_ref())?;

        let mut iter = children.get(&block)?;

        while let Some(child) = iter.next() {
          queue.push((child.to_vec(), height + 1));
        }
      }

      write.commit()?;
    }

    Ok(())
  }

  pub(crate) fn block(&self, height: u64) -> Result<Option<Block>> {
    let tx = self.database.begin_read()?;

    let heights_to_hashes: ReadOnlyTable<u64, [u8]> = tx.open_table(HEIGHTS_TO_HASHES)?;

    match heights_to_hashes.get(&height)? {
      None => return Ok(None),
      Some(guard) => {
        let hash = guard.to_value();

        let offsets: ReadOnlyTable<[u8], u64> = tx.open_table(BLOCK_OFFSETS)?;
        let offset = offsets.get(hash)?.unwrap().to_value() as usize;

        let blocks = fs::read(self.blocksdir.join("blk00000.dat"))?;

        Ok(Some(Self::decode_block_at(&blocks, offset)?))
      }
    }
  }

  fn block_range_at(blocks: &[u8], offset: usize) -> Result<Range<usize>> {
    assert_eq!(&blocks[offset..offset + 4], &[0xf9, 0xbe, 0xb4, 0xd9]);
    let offset = offset + 4;

    let len = u32::from_le_bytes(blocks[offset..offset + 4].try_into()?) as usize;
    let offset = offset + 4;

    Ok(offset..offset + len)
  }

  fn decode_block_at(blocks: &[u8], offset: usize) -> Result<Block> {
    Ok(Block::consensus_decode(
      &blocks[Self::block_range_at(blocks, offset)?],
    )?)
  }

  pub(crate) fn list(&self, outpoint: OutPoint) -> Result<Vec<(u64, u64)>> {
    let rtx = self.database.begin_read()?;
    let utxords: ReadOnlyTable<[u8], [u8]> = rtx.open_table(UTXORDS)?;
    let mut key = Vec::new();
    outpoint.consensus_encode(&mut key)?;
    let ordinal_ranges = utxords.get(key.as_slice())?.unwrap();
    let mut output = Vec::new();
    for chunk in ordinal_ranges.to_value().chunks_exact(16) {
      let start = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
      let end = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
      output.push((start, end));
    }
    Ok(output)
  }
}
