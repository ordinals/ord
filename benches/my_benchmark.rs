use {
  bitcoin::{
    blockdata::{constants::COIN_VALUE, script},
    consensus::Encodable,
    Block, BlockHeader, Network, OutPoint, Transaction, TxIn, TxOut,
  },
  criterion::{Criterion, SamplingMode},
  std::{env, fs::File, io::Write, path::Path, process::Command, time::Duration},
  tempfile::TempDir,
};

fn bench_function(blocksdir: &Path) {
  let tempdir = TempDir::new().unwrap();
  let binary = env::current_dir().unwrap().join("target/release/ord");
  assert!(Command::new(binary)
    .arg("index")
    .arg("--blocksdir")
    .arg(blocksdir)
    .current_dir(tempdir.path())
    .output()
    .unwrap()
    .status
    .success())
}

fn bench(c: &mut Criterion) {
  let tempdir = TempDir::new().unwrap();
  let blocksdir = tempdir.path().join("blocks");

  std::fs::create_dir(&blocksdir).unwrap();
  let mut blockfile = File::create(blocksdir.join("blk00000.dat")).unwrap();

  let block = Block {
    header: BlockHeader {
      version: 0,
      prev_blockhash: Default::default(),
      merkle_root: Default::default(),
      time: 0,
      bits: 0,
      nonce: 0,
    },
    txdata: vec![Transaction {
      version: 0,
      lock_time: 0,
      input: vec![TxIn {
        previous_output: OutPoint::null(),
        script_sig: script::Builder::new().push_scriptint(0).into_script(),
        sequence: 0,
        witness: vec![],
      }],
      output: vec![TxOut {
        value: 50 * COIN_VALUE,
        script_pubkey: script::Builder::new().into_script(),
      }],
    }],
  };

  let mut encoded = Vec::new();
  block.consensus_encode(&mut encoded).unwrap();
  blockfile
    .write_all(&Network::Bitcoin.magic().to_le_bytes())
    .unwrap();
  blockfile
    .write_all(&(encoded.len() as u32).to_le_bytes())
    .unwrap();
  blockfile.write_all(&encoded).unwrap();

  let mut group = c.benchmark_group("flat-sampling-example");
  group.sampling_mode(SamplingMode::Flat);
  group.bench_function("bench", |b| b.iter(|| bench_function(&blocksdir)));
  group.finish();
}

fn main() {
  let mut criterion = Criterion::default()
    .configure_from_args()
    .measurement_time(Duration::from_secs(10));
  bench(&mut criterion);
  criterion.final_summary();
}
