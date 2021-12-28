use super::*;

#[derive(StructOpt)]
pub enum Arguments {
  FindSatoshi { n: u64 },
}

impl Arguments {
  pub fn run(self) -> Result<()> {
    match self {
      Self::FindSatoshi { n } => {
        assert_eq!(n, 0);

        let blocks =
          fs::read("/Users/rodarmor/Library/Application Support/Bitcoin/blocks/blk00000.dat")?;

        assert_eq!(&blocks[0..4], &[0xf9, 0xbe, 0xb4, 0xd9]);

        let len = u32::from_le_bytes(blocks[4..8].try_into()?);

        let block = &blocks[8..8 + len as usize];

        let block = Block::consensus_decode(block)?;

        assert_eq!(block, genesis_block(Network::Bitcoin));

        println!("{}:0", block.txdata[0].txid());

        Ok(())
      }
    }
  }
}
