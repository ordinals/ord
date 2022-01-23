use {
  bitcoin::{
    blockdata::constants::{genesis_block, COIN_VALUE, MAX_SEQUENCE},
    blockdata::script,
    consensus::Encodable,
    hashes::sha256d,
    {Block, BlockHeader, Network, OutPoint, Transaction, TxIn, TxOut},
  },
  executable_path::executable_path,
  std::{
    collections::BTreeSet,
    error::Error,
    fs::{self, File},
    io::{self, Seek, SeekFrom, Write},
    process::Command,
    str,
  },
  tempfile::TempDir,
  unindent::Unindent,
};

mod epochs;
mod find;
mod list;
mod name;
mod range;
mod supply;
mod traits;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

struct Test {
  args: Vec<String>,
  expected_stdout: String,
  expected_stderr: String,
  expected_status: i32,
  ignore_stdout: bool,
  tempdir: TempDir,
}

impl Test {
  fn new() -> Result<Self> {
    Ok(Self {
      args: Vec::new(),
      expected_stdout: String::new(),
      expected_stderr: String::new(),
      expected_status: 0,
      ignore_stdout: false,
      tempdir: TempDir::new()?,
    })
  }

  fn command(self, args: &str) -> Self {
    Self {
      args: args.split_whitespace().map(str::to_owned).collect(),
      ..self
    }
  }

  fn args(self, args: &[&str]) -> Self {
    Self {
      args: self
        .args
        .into_iter()
        .chain(args.iter().cloned().map(str::to_owned))
        .collect(),
      ..self
    }
  }

  fn expected_stdout(self, expected_stdout: impl AsRef<str>) -> Self {
    Self {
      expected_stdout: expected_stdout.as_ref().to_owned(),
      ..self
    }
  }

  fn expected_stderr(self, expected_stderr: &str) -> Self {
    Self {
      expected_stderr: expected_stderr.to_owned(),
      ..self
    }
  }

  fn expected_status(self, expected_status: i32) -> Self {
    Self {
      expected_status,
      ..self
    }
  }

  fn ignore_stdout(self) -> Self {
    Self {
      ignore_stdout: true,
      ..self
    }
  }

  fn run(self) -> Result {
    self.run_with_stdout().map(|_| ())
  }

  fn run_with_stdout(self) -> Result<String> {
    let blocksdir = self.tempdir.path().join("blocks");
    fs::create_dir(&blocksdir)?;
    populate_blockfile(File::create(blocksdir.join("blk00000.dat"))?, 1)?;

    let output = Command::new(executable_path("ord"))
      .current_dir(&self.tempdir)
      .args(self.args)
      .output()?;

    let stderr = str::from_utf8(&output.stderr)?;

    if output.status.code() != Some(self.expected_status) {
      panic!("Test failed: {}\n{}", output.status, stderr);
    }

    assert_eq!(stderr, self.expected_stderr);

    let stdout = str::from_utf8(&output.stdout)?;

    if !self.ignore_stdout {
      assert_eq!(stdout, self.expected_stdout);
    }

    Ok(stdout.to_owned())
  }
}

fn generate_coinbase_transaction(height: usize) -> Transaction {
  // Base
  let mut ret = Transaction {
    version: 1,
    lock_time: 0,
    input: vec![],
    output: vec![],
  };

  // Inputs
  let in_script = script::Builder::new()
    .push_scriptint(height as i64)
    .into_script();
  ret.input.push(TxIn {
    previous_output: OutPoint::null(),
    script_sig: in_script,
    sequence: MAX_SEQUENCE,
    witness: vec![],
  });

  // Outputs
  let out_script = script::Builder::new().into_script();
  ret.output.push(TxOut {
    value: 50 * COIN_VALUE,
    script_pubkey: out_script,
  });

  // end
  ret
}

fn generate_spending_transaction(previous_output: OutPoint) -> Transaction {
  // Base
  let mut ret = Transaction {
    version: 1,
    lock_time: 0,
    input: vec![],
    output: vec![],
  };

  // Inputs
  let in_script = script::Builder::new().into_script();
  ret.input.push(TxIn {
    script_sig: in_script,
    sequence: MAX_SEQUENCE,
    witness: vec![],
    previous_output,
  });

  // Outputs
  let out_script = script::Builder::new().into_script();
  ret.output.push(TxOut {
    value: 50 * COIN_VALUE,
    script_pubkey: out_script,
  });

  // end
  ret
}

fn serialize_block(output: &mut File, block: &Block) -> io::Result<()> {
  output.write_all(&[0xf9, 0xbe, 0xb4, 0xd9])?;
  let size_field = output.stream_position()?;
  output.write_all(&[0u8; 4])?;
  let size = block.consensus_encode(&mut *output)?;
  output.seek(SeekFrom::Start(size_field))?;
  output.write_all(&(size as u32).to_le_bytes())?;
  output.seek(SeekFrom::Current(size as i64))?;
  Ok(())
}

fn populate_blockfile(mut output: File, height: usize) -> io::Result<()> {
  let genesis = genesis_block(Network::Bitcoin);
  serialize_block(&mut output, &genesis)?;

  let mut prev_block = genesis.clone();
  for _ in 1..=height {
    let tx = generate_coinbase_transaction(height);
    let hash: sha256d::Hash = tx.txid().into();
    let merkle_root = hash.into();
    let block = Block {
      header: BlockHeader {
        version: 0,
        prev_blockhash: prev_block.block_hash(),
        merkle_root,
        time: 0,
        bits: 0,
        nonce: 0,
      },
      txdata: vec![
        tx,
        generate_spending_transaction(OutPoint {
          txid: prev_block.txdata[0].txid(),
          vout: 0,
        }),
      ],
    };
    serialize_block(&mut output, &block)?;
    prev_block = block;
  }

  Ok(())
}
