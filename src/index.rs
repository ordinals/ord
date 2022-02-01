use super::*;

pub(crate) struct Index {
  blocksdir: PathBuf,
  database: Database,
}

impl Index {
  const HASH_TO_BLOCK: &'static str = "HASH_TO_BLOCK";
  const HASH_TO_CHILDREN: &'static str = "HASH_TO_CHILDREN";
  const HASH_TO_HEIGHT: &'static str = "HASH_TO_HEIGHT";
  const HEIGHT_TO_HASH: &'static str = "HEIGHT_TO_HASH";
  const OUTPOINT_TO_ORDINAL_RANGES: &'static str = "OUTPOINT_TO_ORDINAL_RANGES";

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
      database: unsafe { Database::open("index.redb", 50 << 30)? },
      blocksdir,
    };

    index.index_blockfile()?;

    index.index_ranges()?;

    Ok(index)
  }

  fn index_ranges(&self) -> Result {
    let mut height = 0;
    while let Some(block) = self.block(height)? {
      let wtx = self.database.begin_write()?;
      let mut outpoint_to_ordinal_ranges: Table<[u8], [u8]> =
        wtx.open_table(Self::OUTPOINT_TO_ORDINAL_RANGES)?;

      let mut coinbase_inputs = VecDeque::new();

      let h = Height(height);
      if let Some(start) = h.starting_ordinal() {
        coinbase_inputs.push_front((start.n(), (start + h.subsidy()).n()));
      }

      for tx in block.txdata.iter().skip(1) {
        let mut input_ordinal_ranges = VecDeque::new();
        for input in &tx.input {
          let mut key = Vec::new();
          input.previous_output.consensus_encode(&mut key)?;
          let ordinal_ranges = outpoint_to_ordinal_ranges
            .get(key.as_slice())?
            .ok_or("Could not find outpoint in index")?;

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
            let range = input_ordinal_ranges
              .pop_front()
              .ok_or("Found transaction with outputs but no inputs")?;
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
          outpoint_to_ordinal_ranges.insert(&key, &ordinals)?;
        }
        coinbase_inputs.extend(&input_ordinal_ranges);
      }

      if let Some(tx) = block.txdata.first() {
        for (vout, output) in tx.output.iter().enumerate() {
          let mut ordinals = Vec::new();
          let mut remaining = output.value;
          while remaining > 0 {
            let range = coinbase_inputs
              .pop_front()
              .ok_or("Insufficient inputs for coinbase transaction outputs")?;
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
          outpoint_to_ordinal_ranges.insert(&key, &ordinals)?;
        }
      }

      wtx.commit()?;
      height += 1;
    }

    Ok(())
  }

  fn index_blockfile(&self) -> Result {
    let mut blockfiles = 0;
    loop {
      match File::open(self.blocksdir.join(format!("blk{:05}.dat", blockfiles))) {
        Ok(_) => {}
        Err(err) => {
          if err.kind() == io::ErrorKind::NotFound {
            break;
          } else {
            return Err(err.into());
          }
        }
      }
      blockfiles += 1;
    }

    log::info!("Indexing {} blockfiles…", blockfiles);

    for i in 0.. {
      let blocks = match fs::read(self.blocksdir.join(format!("blk{:05}.dat", i))) {
        Ok(blocks) => blocks,
        Err(err) => {
          if err.kind() == io::ErrorKind::NotFound {
            break;
          } else {
            return Err(err.into());
          }
        }
      };

      let tx = self.database.begin_write()?;

      let mut hash_to_children: MultimapTable<[u8], [u8]> =
        tx.open_multimap_table(Self::HASH_TO_CHILDREN)?;

      let mut hash_to_block: Table<[u8], [u8]> = tx.open_table(Self::HASH_TO_BLOCK)?;

      let mut offset = 0;

      let mut count = 0;

      loop {
        if offset == blocks.len() {
          break;
        }

        let range = Self::block_range_at(&blocks, offset)?;

        let block = Block::consensus_decode(&blocks[range.clone()])?;

        hash_to_children.insert(&block.header.prev_blockhash, &block.block_hash())?;

        hash_to_block.insert(&block.block_hash(), &blocks[range.clone()])?;

        offset = range.end;

        count += 1;
      }

      log::info!("{}/{}: Processed {} blocks…", i + 1, blockfiles, count);

      tx.commit()?;
    }

    {
      let write = self.database.begin_write()?;

      let mut hash_to_height: Table<[u8], u64> = write.open_table(Self::HASH_TO_HEIGHT)?;
      let mut height_to_hash: Table<u64, [u8]> = write.open_table(Self::HEIGHT_TO_HASH)?;

      hash_to_height.insert(genesis_block(Network::Bitcoin).block_hash().deref(), &0)?;
      height_to_hash.insert(&0, genesis_block(Network::Bitcoin).block_hash().deref())?;

      let read = self.database.begin_read()?;

      let hash_to_children: ReadOnlyMultimapTable<[u8], [u8]> =
        read.open_multimap_table(Self::HASH_TO_CHILDREN)?;

      let mut queue = vec![(
        genesis_block(Network::Bitcoin)
          .block_hash()
          .deref()
          .to_vec(),
        0,
      )];

      while let Some((block, height)) = queue.pop() {
        hash_to_height.insert(block.as_ref(), &height)?;
        height_to_hash.insert(&height, block.as_ref())?;

        let mut iter = hash_to_children.get(&block)?;

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

    let heights_to_hash: ReadOnlyTable<u64, [u8]> = tx.open_table(Self::HEIGHT_TO_HASH)?;

    match heights_to_hash.get(&height)? {
      None => Ok(None),
      Some(guard) => {
        let hash = guard.to_value();

        let hash_to_block: ReadOnlyTable<[u8], [u8]> = tx.open_table(Self::HASH_TO_BLOCK)?;

        Ok(Some(Block::consensus_decode(
          hash_to_block
            .get(hash)?
            .ok_or("Could not find block in index")?
            .to_value(),
        )?))
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

  pub(crate) fn list(&self, outpoint: OutPoint) -> Result<Vec<(u64, u64)>> {
    let rtx = self.database.begin_read()?;
    let outpoint_to_ordinal_ranges: ReadOnlyTable<[u8], [u8]> =
      rtx.open_table(Self::OUTPOINT_TO_ORDINAL_RANGES)?;

    let mut key = Vec::new();
    outpoint.consensus_encode(&mut key)?;

    let ordinal_ranges = outpoint_to_ordinal_ranges
      .get(key.as_slice())?
      .ok_or("Could not find outpoint in index")?;

    let mut output = Vec::new();
    for chunk in ordinal_ranges.to_value().chunks_exact(16) {
      let start = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
      let end = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
      output.push((start, end));
    }

    Ok(output)
  }
}
