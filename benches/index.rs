use {
  bitcoin::{
    blockdata::{constants::COIN_VALUE, script},
    consensus::Encodable,
    Block, BlockHeader, Network, OutPoint, Transaction, TxIn, TxOut,
  },
  criterion::{Criterion, SamplingMode},
  std::{env, fs::File, io::Seek, io::Write, path::Path, process::Command, time::Duration},
  tempfile::TempDir,
};

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn index(blocksdir: &Path) -> Result {
  let tempdir = TempDir::new()?;
  let binary = env::current_dir()?.join("target/release/ord");
  assert!(Command::new(binary)
    .arg("index")
    .arg("--blocksdir")
    .arg(blocksdir)
    .current_dir(tempdir.path())
    .status()?
    .success());
  Ok(())
}

fn main() -> Result {
  let mut criterion = Criterion::default()
    .configure_from_args()
    .measurement_time(Duration::from_secs(60));

  let tempdir = TempDir::new()?;
  let blocksdir = tempdir.path().join("blocks");

  std::fs::create_dir(&blocksdir)?;
  let mut blockfile = File::create(blocksdir.join("blk00000.dat"))?;

  let mut blocks = Vec::new();

  for height in 0..1000 {
    blocks.push(Block {
      header: BlockHeader {
        bits: 0,
        merkle_root: Default::default(),
        nonce: 0,
        prev_blockhash: blocks.last().map(Block::block_hash).unwrap_or_default(),
        time: 0,
        version: 0,
      },
      txdata: vec![Transaction {
        input: vec![TxIn {
          previous_output: OutPoint::null(),
          script_sig: script::Builder::new().push_scriptint(height).into_script(),
          sequence: 0,
          witness: vec![],
        }],
        lock_time: 0,
        output: vec![TxOut {
          value: 50 * COIN_VALUE,
          script_pubkey: script::Builder::new().into_script(),
        }],
        version: 0,
      }],
    });
  }

  for block in blocks {
    let mut encoded = Vec::new();
    block.consensus_encode(&mut encoded)?;
    blockfile.write_all(&Network::Bitcoin.magic().to_le_bytes())?;
    blockfile.write_all(&(encoded.len() as u32).to_le_bytes())?;
    blockfile.write_all(&encoded)?;
  }

  blockfile.flush()?;

  eprintln!("Blockfile is {} bytes", blockfile.stream_position()?);

  let mut group = criterion.benchmark_group("index");
  group.sampling_mode(SamplingMode::Flat);
  group.bench_function("1000 blocks", |b| b.iter(|| index(&blocksdir)));
  group.finish();

  criterion.final_summary();

  Ok(())
}
