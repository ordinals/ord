use {
  crate::rpc_server::RpcServer,
  bitcoin::{
    blockdata::constants::COIN_VALUE, blockdata::script, consensus::Encodable, Block, BlockHash,
    BlockHeader, OutPoint, Transaction, TxIn, TxOut,
  },
  executable_path::executable_path,
  regex::Regex,
  std::{
    collections::BTreeSet,
    error::Error,
    process::Command,
    str,
    sync::{Arc, Mutex},
    thread,
  },
  tempfile::TempDir,
  unindent::Unindent,
};

mod epochs;
mod find;
mod index;
mod info;
mod list;
mod name;
mod range;
mod rpc_server;
mod supply;
mod traits;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

enum Expected {
  String(String),
  Regex(Regex),
  Ignore,
}

struct Output {
  calls: Vec<String>,
  stdout: String,
  tempdir: TempDir,
}

struct CoinbaseOptions {
  include_coinbase_transaction: bool,
  include_height: bool,
  subsidy: u64,
}

impl Default for CoinbaseOptions {
  fn default() -> Self {
    Self {
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
  blocks: Vec<Block>,
  expected_status: i32,
  expected_stderr: String,
  expected_stdout: Expected,
  tempdir: TempDir,
}

impl Test {
  fn new() -> Result<Self> {
    Ok(Self::with_tempdir(TempDir::new()?))
  }

  fn with_tempdir(tempdir: TempDir) -> Self {
    Self {
      args: Vec::new(),
      blocks: Vec::new(),
      expected_status: 0,
      expected_stderr: String::new(),
      expected_stdout: Expected::String(String::new()),
      tempdir,
    }
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
      expected_stdout: Expected::String(expected_stdout.as_ref().to_owned()),
      ..self
    }
  }

  fn stdout_regex(self, expected_stdout: impl AsRef<str>) -> Self {
    Self {
      expected_stdout: Expected::Regex(
        Regex::new(&format!("^{}$", expected_stdout.as_ref())).unwrap(),
      ),
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
      expected_stdout: Expected::Ignore,
      ..self
    }
  }

  fn run(self) -> Result {
    self.output().map(|_| ())
  }

  fn output(self) -> Result<Output> {
    for (b, block) in self.blocks.iter().enumerate() {
      for (t, transaction) in block.txdata.iter().enumerate() {
        eprintln!("{b}.{t}: {}", transaction.txid());
      }
    }

    let (close_handle, calls, port) = RpcServer::spawn(&self.blocks);

    let output = Command::new(executable_path("ord"))
      .current_dir(&self.tempdir)
      .arg(format!("--rpc-url=http://127.0.0.1:{port}"))
      .args(self.args)
      .output()?;

    close_handle.close();

    let stdout = str::from_utf8(&output.stdout)?;
    let stderr = str::from_utf8(&output.stderr)?;

    if output.status.code() != Some(self.expected_status) {
      panic!(
        "Test failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status, stdout, stderr
      );
    }

    let re = Regex::new(r"(?m)^\[.*\n")?;

    for m in re.find_iter(stderr) {
      print!("{}", m.as_str())
    }

    assert_eq!(re.replace_all(stderr, ""), self.expected_stderr);

    match self.expected_stdout {
      Expected::String(expected_stdout) => assert_eq!(stdout, expected_stdout),
      Expected::Regex(expected_stdout) => assert!(
        expected_stdout.is_match(stdout),
        "stdout did not match regex: {}",
        stdout
      ),
      Expected::Ignore => {}
    }

    let calls = calls.lock().unwrap().clone();

    Ok(Output {
      stdout: stdout.to_string(),
      tempdir: self.tempdir,
      calls,
    })
  }

  fn block(self) -> Self {
    self.block_with_coinbase(CoinbaseOptions::default())
  }

  fn block_with_coinbase(mut self, coinbase: CoinbaseOptions) -> Self {
    self.blocks.push(Block {
      header: BlockHeader {
        version: 0,
        prev_blockhash: self
          .blocks
          .last()
          .map(Block::block_hash)
          .unwrap_or_default(),
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
}
