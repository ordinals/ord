use {
  super::*,
  bitcoin::{consensus::Decodable, Block},
  memmap::Mmap,
  redb::Database,
  redb::Table,
};

struct Catalog {
  database: Database,
}

impl Catalog {
  unsafe fn new() -> Result<Self> {
    Ok(Self {
      database: Database::open("catalog.redb".as_ref(), 2usize.pow(40))
        .map_err(|error| format!("{:?}", error))?,
    })
  }

  fn block_file_path(n: u64) -> Result<PathBuf> {
    Ok(
      dirs::home_dir()
        .ok_or("Failed to retrieve home dir.")?
        .join(format!(
          "Library/Application Support/Bitcoin/blocks/blk{:06}.dat",
          n
        )),
    )
  }

  fn blocks(&self) -> Blocks {
    Blocks {
      next: 0,
      catalog: self,
    }
  }

  fn block(&self, height: u64) -> Result<Option<Block>> {
    let blocks: Table<u64, [u8]> = self
      .database
      .open_table("blocks".as_ref())
      .map_err(|error| format!("{:?}", error))?;

    let read = blocks
      .read_transaction()
      .map_err(|error| format!("{:?}", error))?;

    if let Some(guard) = read.get(&height).map_err(|error| format!("{:?}", error))? {
      return Ok(Some(Block::consensus_decode(guard.to_value())?));
    }

    todo!()
  }
}

// for n in 0.. {
//       let block_file_path = Self::block_file_path(n)?;

//       let file = match File::open(block_file_path) {
//         Ok(file) => file,
//         Err(err) if err.kind() == io::ErrorKind::NotFound => {
//           return Ok(());
//         }
//         Err(err) => return Err(err.into()),
//       };

//       todo!()
//     }

//     Ok(())
//   }

struct Blocks<'a> {
  next: u64,
  catalog: &'a Catalog,
}

impl<'a> Iterator for Blocks<'a> {
  type Item = Result<Block>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.catalog.block(self.next) {
      Err(err) => Some(Err(err)),
      Ok(None) => None,
      Ok(Some(block)) => {
        self.next += 1;
        Some(Ok(block))
      }
    }
  }
}

pub fn run() -> Result<()> {
  let catalog = unsafe { Catalog::new()? };

  for block in catalog.blocks() {}

  let client = client::initialize()?;

  let tip_height = client
    .get_block_header_info(&client.get_best_block_hash()?)?
    .height as u64;

  eprintln!("Scanning for atoms up to height {}…", tip_height);

  let mut atoms: BTreeMap<OutPoint, u64> = BTreeMap::new();

  for height in 0..tip_height {
    eprintln!("Scanning for atoms in block {}…", height);
    let hash = client.get_block_hash(height)?;

    let block = client.get_block(&hash)?;

    for (i, transaction) in block.txdata.iter().enumerate() {
      let txid = transaction.txid();
      if i == 0 {
        atoms.insert(OutPoint { txid, vout: 0 }, height);
      } else {
        let mut transferred = transaction
          .input
          .iter()
          .map(|txin| atoms.remove_entry(&txin.previous_output))
          .flatten()
          .collect::<Vec<(OutPoint, u64)>>();

        if transferred.is_empty() {
          continue;
        }

        eprintln!(
          "Transferring {} atoms: {:?}",
          transferred.len(),
          transferred,
        );

        let total = transaction
          .output
          .iter()
          .map(|txout| txout.value)
          .sum::<u64>();

        let mut pending = 0;
        for (vout, output) in transaction.output.iter().enumerate() {
          let vout = vout as u32;

          pending += output.value * transferred.len() as u64;

          while pending >= total {
            let (old_outpoint, atom) = transferred.remove(0);
            let new_outpoint = OutPoint { vout, txid };
            eprintln!(
              "Transferring atom {} from {} to {}",
              atom, old_outpoint, new_outpoint
            );
            atoms.insert(new_outpoint, atom);
            pending -= total;
          }
        }

        assert!(transferred.is_empty());
      }
    }
  }

  Ok(())
}
