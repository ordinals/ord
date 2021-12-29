use super::*;

#[derive(StructOpt)]
pub enum Arguments {
  FindSatoshi { n: u64, at_height: u64 },
}

impl Arguments {
  pub fn run(self) -> Result<()> {
    match self {
      Self::FindSatoshi { n, at_height } => {
        let tempdir = tempfile::tempdir()?;

        let db = unsafe {
          Database::open(tempdir.path().join("bitcoin.redb"), 4096 * 1024 * 1024).unwrap()
        };

        {
          let tx = db.begin_write()?;

          let mut parents: Table<[u8], [u8]> = tx.open_table("parents")?;

          let blocks =
            fs::read("/Users/rodarmor/Library/Application Support/Bitcoin/blocks/blk00000.dat")?;

          let mut i = 0;

          loop {
            if i == blocks.len() {
              break;
            }

            assert_eq!(&blocks[i..i + 4], &[0xf9, 0xbe, 0xb4, 0xd9]);
            let len = u32::from_le_bytes(blocks[i + 4..i + 8].try_into()?) as usize;
            let bytes = &blocks[i + 8..i + 8 + len];
            let block = Block::consensus_decode(bytes)?;

            parents.insert(
              &block.block_hash().deref(),
              &block.header.prev_blockhash.deref(),
            )?;

            i += 8 + len;
          }

          tx.commit()?;
        }

        {
          let write = db.begin_write()?;

          let mut heights: Table<[u8], u64> = write.open_table("heights")?;

          heights.insert(genesis_block(Network::Bitcoin).block_hash().deref(), &0);

          let read = db.begin_read()?;

          let parents: ReadOnlyTable<[u8], [u8]> = read.open_table("parents")?;

          let mut range = parents.get_range(..)?;

          while let Some(accessor) = range.next() {
            height(&parents, &mut heights, accessor.key())?;
          }
        }

        Ok(())
      }
    }
  }
}

fn height(
  parents: &ReadOnlyTable<[u8], [u8]>,
  heights: &mut Table<[u8], u64>,
  block: BlockHash,
) -> Result<()> {
  let parent = parents.get(&block)?.unwrap();

  if heights.get(parent.to_value())?.is_none() {
    height(parents, heights, parent)?;
  }

  heights.insert(block, &(heights.get(parent)?.to_value() + 1))?
}
