use super::*;

pub(crate) struct Index {
  blocksdir: PathBuf,
  database: Database,
}

impl Index {
  const HASH_TO_CHILDREN: &'static str = "HASH_TO_CHILDREN";
  const HASH_TO_HEIGHT: &'static str = "HASH_TO_HEIGHT";
  const HASH_TO_LOCATION: &'static str = "HASH_TO_LOCATION";
  const HEIGHT_TO_HASH: &'static str = "HEIGHT_TO_HASH";
  const OUTPOINT_TO_ORDINAL_RANGES: &'static str = "OUTPOINT_TO_ORDINAL_RANGES";

  pub(crate) fn new(blocksdir: Option<&Path>, index_size: Option<usize>) -> Result<Self> {
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
      database: unsafe { Database::open("index.redb", index_size.unwrap_or(1 << 20))? },
      blocksdir,
    };

    index.index_blockfiles()?;

    index.index_heights()?;

    index.index_ranges()?;

    Ok(index)
  }

  fn index_ranges(&self) -> Result {
    log::info!("Indexing ranges…");

    let mut height = 0;
    while let Some(block) = self.block(height)? {
      let wtx = self.database.begin_write()?;
      let mut outpoint_to_ordinal_ranges: Table<[u8], [u8]> =
        wtx.open_table(Self::OUTPOINT_TO_ORDINAL_RANGES)?;

      let mut coinbase_inputs = VecDeque::new();

      let h = Height(height);
      if h.subsidy() > 0 {
        let start = h.starting_ordinal();
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
              let middle = range.0 + remaining;
              input_ordinal_ranges.push_front((middle, range.1));
              (range.0, middle)
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
              let middle = range.0 + remaining;
              coinbase_inputs.push_front((middle, range.1));
              (range.0, middle)
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

  fn blockfile_path(&self, i: u64) -> PathBuf {
    self.blocksdir.join(format!("blk{:05}.dat", i))
  }

  fn index_blockfiles(&self) -> Result {
    let blockfiles = (0..)
      .map(|i| self.blockfile_path(i))
      .take_while(|path| path.is_file())
      .count();

    log::info!("Indexing {} blockfiles…", blockfiles);

    for i in 0.. {
      let path = self.blockfile_path(i);

      if !path.is_file() {
        break;
      }

      let blocks = unsafe { Mmap::map(&File::open(path)?)? };

      let tx = self.database.begin_write()?;

      let mut hash_to_children: MultimapTable<[u8], [u8]> =
        tx.open_multimap_table(Self::HASH_TO_CHILDREN)?;

      let mut hash_to_location: Table<[u8], u64> = tx.open_table(Self::HASH_TO_LOCATION)?;

      let mut offset = 0;

      let mut count = 0;

      let mut genesis = false;

      loop {
        if offset == blocks.len() {
          break;
        }

        let rest = &blocks[offset..];

        if rest.starts_with(&[0, 0, 0, 0]) {
          break;
        }

        let block = Self::extract_block(rest)?;

        let header = BlockHeader::consensus_decode(&block[0..80])?;
        let hash = header.block_hash();

        if header.prev_blockhash == Default::default() {
          if genesis {
            return Err("Duplicate genesis block found".into());
          }

          let mut hash_to_height: Table<[u8], u64> = tx.open_table(Self::HASH_TO_HEIGHT)?;
          let mut height_to_hash: Table<u64, [u8]> = tx.open_table(Self::HEIGHT_TO_HASH)?;

          hash_to_height.insert(&hash, &0)?;
          height_to_hash.insert(&0, &hash)?;

          genesis = true;
        }

        hash_to_children.insert(&header.prev_blockhash, &hash)?;

        hash_to_location.insert(&hash, &((i as u64) << 32 | offset as u64))?;

        offset = offset + 8 + block.len();

        count += 1;
      }

      log::info!("{}/{}: Processed {} blocks…", i + 1, blockfiles + 1, count);

      tx.commit()?;
    }

    Ok(())
  }

  fn index_heights(&self) -> Result {
    log::info!("Indexing heights…");

    let write = self.database.begin_write()?;

    let read = self.database.begin_read()?;

    let hash_to_children: ReadOnlyMultimapTable<[u8], [u8]> =
      read.open_multimap_table(Self::HASH_TO_CHILDREN)?;

    let mut hash_to_height: Table<[u8], u64> = write.open_table(Self::HASH_TO_HEIGHT)?;
    let mut height_to_hash: Table<u64, [u8]> = write.open_table(Self::HEIGHT_TO_HASH)?;

    let mut queue = vec![(
      height_to_hash
        .get(&0)?
        .ok_or("Could not find genesis block in index")?
        .to_value()
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

    Ok(())
  }

  fn extract_block(blocks: &[u8]) -> Result<&[u8]> {
    let magic = &blocks[0..4];
    if magic != Network::Bitcoin.magic().to_le_bytes() {
      return Err(format!("Unknown magic bytes: {:?}", magic).into());
    }

    let len = u32::from_le_bytes(blocks[4..8].try_into()?) as usize;

    Ok(&blocks[8..8 + len])
  }

  pub(crate) fn block(&self, height: u64) -> Result<Option<Block>> {
    let tx = self.database.begin_read()?;

    let heights_to_hash: ReadOnlyTable<u64, [u8]> = tx.open_table(Self::HEIGHT_TO_HASH)?;

    match heights_to_hash.get(&height)? {
      None => Ok(None),
      Some(guard) => {
        let hash = guard.to_value();

        let hash_to_location: ReadOnlyTable<[u8], u64> = tx.open_table(Self::HASH_TO_LOCATION)?;

        let location = hash_to_location
          .get(hash)?
          .ok_or("Could not find block location in index")?
          .to_value();

        let path = self.blockfile_path(location >> 32);

        let offset = (location & 0xFFFFFFFF) as usize;

        let blocks = unsafe { Mmap::map(&File::open(path)?)? };

        let bytes = Self::extract_block(&blocks[offset..])?;

        Ok(Some(Block::consensus_decode(bytes)?))
      }
    }
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
