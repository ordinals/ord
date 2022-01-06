use super::*;

const DESCENDENTS: &str = "descendents";
const HEIGHTS: &str = "heights";
const BLOCK_OFFSETS: &str = "block_offsets";
const HASHES: &str = "hashes";

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
          Database::open(tempdir.path().join("bitcoin.redb"), 4096 * 1024 * 1024 * 10).unwrap()
        };

        {
          let tx = db.begin_write()?;

          let mut descendents: MultimapTable<[u8], [u8]> = tx.open_multimap_table(DESCENDENTS)?;

          let mut block_offsets: Table<[u8], u64> = tx.open_table(BLOCK_OFFSETS)?;

          let blocks =
            fs::read("/Users/rodarmor/Library/Application Support/Bitcoin/blocks/blk00000.dat")?;

          let mut i = 0;

          let mut count = 0;

          loop {
            if i == blocks.len() {
              break;
            }

            let offset = i;

            assert_eq!(&blocks[i..i + 4], &[0xf9, 0xbe, 0xb4, 0xd9]);
            i += 4;

            let len = u32::from_le_bytes(blocks[i..i + 4].try_into()?) as usize;
            i += 4;

            let bytes = &blocks[i..i + len];
            i += len;

            let block = Block::consensus_decode(bytes)?;

            descendents.insert(
              &block.header.prev_blockhash.deref(),
              &block.block_hash().deref(),
            )?;

            block_offsets.insert(block.block_hash().deref(), &(offset as u64))?;

            count += 1;
          }

          eprintln!("Inserted {} blocks…", count);

          tx.commit()?;
        }

        {
          let write = db.begin_write()?;

          let mut heights: Table<[u8], u64> = write.open_table(HEIGHTS)?;
          let mut hashes: Table<u64, [u8]> = write.open_table(HASHES)?;

          heights.insert(genesis_block(Network::Bitcoin).block_hash().deref(), &0)?;
          hashes.insert(&0, genesis_block(Network::Bitcoin).block_hash().deref())?;

          let read = db.begin_read()?;

          let descendents: ReadOnlyMultimapTable<[u8], [u8]> =
            read.open_multimap_table(DESCENDENTS)?;

          let mut queue = vec![(
            genesis_block(Network::Bitcoin)
              .block_hash()
              .deref()
              .to_vec(),
            0,
          )];

          while let Some((block, height)) = queue.pop() {
            heights.insert(block.as_ref(), &height)?;
            hashes.insert(&height, block.as_ref())?;

            let mut iter = descendents.get(&block)?;

            while let Some(descendent) = iter.next() {
              queue.push((descendent.to_vec(), height + 1));
            }
          }

          write.commit()?;
        }

        let height = n / (50 * 100_000_000);
        assert!(height < 100);
        assert!(at_height == height);

        let tx = db.begin_read()?;

        let hashes: ReadOnlyTable<u64, [u8]> = tx.open_table(HASHES)?;
        let guard = hashes.get(&height)?.unwrap();
        let hash = guard.to_value();

        let offsets: ReadOnlyTable<[u8], u64> = tx.open_table(BLOCK_OFFSETS)?;
        let mut i = offsets.get(hash)?.unwrap().to_value() as usize;

        if i == 1 {
          i = 0;
        }

        let blocks =
          fs::read("/Users/rodarmor/Library/Application Support/Bitcoin/blocks/blk00000.dat")?;

        assert_eq!(&blocks[i..i + 4], &[0xf9, 0xbe, 0xb4, 0xd9]);
        i += 4;

        let len = u32::from_le_bytes(blocks[i..i + 4].try_into()?) as usize;
        i += 4;

        let bytes = &blocks[i..i + len];

        let block = Block::consensus_decode(bytes)?;

        let position = n % (50 * 100_000_000);

        let mut n = 0;
        for (i, output) in block.txdata[0].output.iter().enumerate() {
          if n + output.value >= position {
            println!("{}:{}", block.txdata[0].txid(), i);
            break;
          }
          n += output.value;
        }

        Ok(())
      }
    }
  }
}
