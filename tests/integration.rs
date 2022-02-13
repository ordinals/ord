use {
  bitcoin::{
    blockdata::constants::COIN_VALUE,
    blockdata::script,
    consensus::Encodable,
    {Block, BlockHeader, Network, OutPoint, Transaction, TxIn, TxOut},
  },
  executable_path::executable_path,
  regex::Regex,
  std::{
    collections::BTreeSet,
    error::Error,
    fs::{self, File},
    io::{self, Write},
    iter,
    process::Command,
    str,
  },
  tempfile::TempDir,
  unindent::Unindent,
};

mod epochs;
mod find;
mod index;
mod list;
mod name;
mod range;
mod supply;
mod traits;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

struct Output {
  stdout: String,
  tempdir: TempDir,
}

struct CoinbaseOptions {
  default_prev_blockhash: bool,
  include_coinbase_transaction: bool,
  include_height: bool,
  subsidy: u64,
}

impl Default for CoinbaseOptions {
  fn default() -> Self {
    Self {
      default_prev_blockhash: false,
      include_coinbase_transaction: true,
      include_height: true,
      subsidy: 50 * COIN_VALUE,
    }
  }
}

struct TransactionOptions<'a> {
  slots: &'a [(usize, usize, usize)],
  output_count: usize,
  fee: u64,
}

struct Test {
  args: Vec<String>,
  blockfiles: Vec<usize>,
  blocks: Vec<Block>,
  expected_status: i32,
  expected_stderr: String,
  expected_stdout: String,
  ignore_stdout: bool,
  tempdir: TempDir,
  reverse_blockfiles: bool,
}

impl Test {
  fn new() -> Result<Self> {
    Ok(Self {
      args: Vec::new(),
      blockfiles: Vec::new(),
      blocks: Vec::new(),
      expected_status: 0,
      expected_stderr: String::new(),
      expected_stdout: String::new(),
      ignore_stdout: false,
      tempdir: TempDir::new()?,
      reverse_blockfiles: false,
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

  fn reverse_blockfiles(self) -> Self {
    Self {
      reverse_blockfiles: true,
      ..self
    }
  }

  fn run(self) -> Result {
    self.output().map(|_| ())
  }

  fn output(self) -> Result<Output> {
    self.create_blockfiles()?;

    let output = Command::new(executable_path("ord"))
      .current_dir(&self.tempdir)
      .args(self.args)
      .output()?;

    let stderr = str::from_utf8(&output.stderr)?;

    if output.status.code() != Some(self.expected_status) {
      panic!("Test failed: {}\n{}", output.status, stderr);
    }

    let re = Regex::new(r"(?m)^\[.*\n")?;

    for m in re.find_iter(stderr) {
      print!("{}", m.as_str())
    }

    assert_eq!(re.replace_all(stderr, ""), self.expected_stderr);

    let stdout = str::from_utf8(&output.stdout)?;

    if !self.ignore_stdout {
      assert_eq!(stdout, self.expected_stdout);
    }

    Ok(Output {
      stdout: stdout.to_string(),
      tempdir: self.tempdir,
    })
  }

  fn block(self) -> Self {
    self.block_with_coinbase(CoinbaseOptions::default())
  }

  fn block_with_coinbase(mut self, coinbase: CoinbaseOptions) -> Self {
    self.blocks.push(Block {
      header: BlockHeader {
        version: 0,
        prev_blockhash: if coinbase.default_prev_blockhash {
          Default::default()
        } else {
          self
            .blocks
            .last()
            .map(Block::block_hash)
            .unwrap_or_default()
        },
        merkle_root: Default::default(),
        time: 0,
        bits: 0,
        nonce: 0,
      },
      txdata: if coinbase.include_coinbase_transaction {
        vec![Transaction {
          version: 0,
          lock_time: 0,
          input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig: if coinbase.include_height {
              script::Builder::new()
                .push_scriptint(self.blocks.len().try_into().unwrap())
                .into_script()
            } else {
              script::Builder::new().into_script()
            },
            sequence: 0,
            witness: vec![],
          }],
          output: vec![TxOut {
            value: coinbase.subsidy,
            script_pubkey: script::Builder::new().into_script(),
          }],
        }]
      } else {
        Vec::new()
      },
    });
    self
  }

  fn transaction(mut self, options: TransactionOptions) -> Self {
    let input_value = options
      .slots
      .iter()
      .map(|slot| self.blocks[slot.0].txdata[slot.1].output[slot.2].value)
      .sum::<u64>();

    let output_value = input_value - options.fee;

    let tx = Transaction {
      version: 0,
      lock_time: 0,
      input: options
        .slots
        .iter()
        .map(|slot| TxIn {
          previous_output: OutPoint {
            txid: self.blocks[slot.0].txdata[slot.1].txid(),
            vout: slot.2 as u32,
          },
          script_sig: script::Builder::new().into_script(),
          sequence: 0,
          witness: vec![],
        })
        .collect(),
      output: vec![
        TxOut {
          value: output_value / options.output_count as u64,
          script_pubkey: script::Builder::new().into_script(),
        };
        options.output_count
      ],
    };

    let block = self.blocks.last_mut().unwrap();

    block
      .txdata
      .first_mut()
      .unwrap()
      .output
      .first_mut()
      .unwrap()
      .value += options.fee;

    block.txdata.push(tx);

    self
  }

  fn blockfile(mut self) -> Self {
    self.blockfiles.push(self.blocks.len());
    self
  }

  fn create_blockfiles(&self) -> io::Result<()> {
    let blocksdir = self.tempdir.path().join("blocks");
    fs::create_dir(&blocksdir)?;

    let mut start = 0;

    for (i, end) in self
      .blockfiles
      .iter()
      .copied()
      .chain(iter::once(self.blocks.len()))
      .enumerate()
    {
      let mut blockfile = File::create(blocksdir.join(format!("blk{:05}.dat", i)))?;

      let blocks = self.blocks[start..end].iter().enumerate();

      let blocks: Box<dyn std::iter::Iterator<Item = (usize, &Block)>> = if self.reverse_blockfiles
      {
        Box::new(blocks.rev())
      } else {
        Box::new(blocks)
      };

      for (bi, block) in blocks {
        let mut encoded = Vec::new();
        block.consensus_encode(&mut encoded)?;
        blockfile.write_all(&Network::Bitcoin.magic().to_le_bytes())?;
        blockfile.write_all(&(encoded.len() as u32).to_le_bytes())?;
        blockfile.write_all(&encoded)?;
        for (ti, tx) in block.txdata.iter().enumerate() {
          eprintln!("{bi}.{ti}: {}", tx.txid());
        }
      }

      start = end;
    }

    Ok(())
  }
}
