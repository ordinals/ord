use super::*;

const CHILDREN: &str = "children";
const HEIGHTS: &str = "heights";
const BLOCK_OFFSETS: &str = "block_offsets";
const HEIGHTS_TO_HASHES: &str = "height_to_hashes";

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
      database: unsafe { Database::open("bitcoin.redb", 4096 * 1024 * 1024 * 10)? },
      blocksdir,
    };

    index.index_blockfile()?;

    Ok(index)
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

  pub(crate) fn block(&self, height: u64) -> Result<Block> {
    let tx = self.database.begin_read()?;

    let heights_to_hashes: ReadOnlyTable<u64, [u8]> = tx.open_table(HEIGHTS_TO_HASHES)?;
    let guard = heights_to_hashes.get(&height)?.unwrap();
    let hash = guard.to_value();

    let offsets: ReadOnlyTable<[u8], u64> = tx.open_table(BLOCK_OFFSETS)?;
    let offset = offsets.get(hash)?.unwrap().to_value() as usize;

    let blocks = fs::read(self.blocksdir.join("blk00000.dat"))?;

    Self::decode_block_at(&blocks, offset)
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
}
