use {
  bitcoin::{
    blockdata::constants::{genesis_block, COIN_VALUE},
    blockdata::script,
    consensus::Encodable,
    {Block, BlockHeader, Network, OutPoint, Transaction, TxIn, TxOut},
  },
  executable_path::executable_path,
  std::{
    collections::BTreeSet,
    error::Error,
    fs::{self, File},
    io::{self, Write},
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
  blocks: Vec<Block>,
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
      blocks: Vec::new(),
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
    self.populate_blocksdir()?;

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

  fn block(mut self) -> Self {
    if self.blocks.is_empty() {
      self.blocks.push(genesis_block(Network::Bitcoin));
    } else {
      self.blocks.push(Block {
        header: BlockHeader {
          version: 0,
          prev_blockhash: self.blocks.last().unwrap().block_hash(),
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
            script_sig: script::Builder::new()
              .push_scriptint(self.blocks.len().try_into().unwrap())
              .into_script(),
            sequence: 0,
            witness: vec![],
          }],
          output: vec![TxOut {
            value: 50 * COIN_VALUE,
            script_pubkey: script::Builder::new().into_script(),
          }],
        }],
      });
    }
    self
  }

  fn block_without_coinbase(mut self) -> Self {
    if self.blocks.is_empty() {
      self.blocks.push(genesis_block(Network::Bitcoin));
    } else {
      self.blocks.push(Block {
        header: BlockHeader {
          version: 0,
          prev_blockhash: self.blocks.last().unwrap().block_hash(),
          merkle_root: Default::default(),
          time: 0,
          bits: 0,
          nonce: 0,
        },
        txdata: Vec::new(),
      });
    }
    self
  }

  fn transaction(mut self, slots: &[(usize, usize, u32)], output_count: u64) -> Self {
    let value = slots
      .iter()
      .map(|slot| self.blocks[slot.0].txdata[slot.1].output[slot.2 as usize].value)
      .sum::<u64>();

    let tx = Transaction {
      version: 0,
      lock_time: 0,
      input: slots
        .iter()
        .map(|slot| TxIn {
          previous_output: OutPoint {
            txid: self.blocks[slot.0].txdata[slot.1].txid(),
            vout: slot.2,
          },
          script_sig: script::Builder::new().into_script(),
          sequence: 0,
          witness: vec![],
        })
        .collect(),
      output: vec![
        TxOut {
          value: value / output_count,
          script_pubkey: script::Builder::new().into_script(),
        };
        output_count.try_into().unwrap()
      ],
    };

    self.blocks.last_mut().unwrap().txdata.push(tx);

    self
  }

  fn populate_blocksdir(&self) -> io::Result<()> {
    let blocksdir = self.tempdir.path().join("blocks");
    fs::create_dir(&blocksdir)?;
    let mut blockfile = File::create(blocksdir.join("blk00000.dat"))?;

    for block in &self.blocks {
      let mut encoded = Vec::new();
      block.consensus_encode(&mut encoded)?;
      blockfile.write_all(&[0xf9, 0xbe, 0xb4, 0xd9])?;
      blockfile.write_all(&(encoded.len() as u32).to_le_bytes())?;
      blockfile.write_all(&encoded)?;
      for tx in &block.txdata {
        eprintln!("{}", tx.txid());
      }
    }

    Ok(())
  }
}
