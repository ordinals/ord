#![allow(clippy::type_complexity)]

use {
  crate::rpc_server::RpcServer,
  bitcoin::{
    blockdata::constants::COIN_VALUE, blockdata::script, consensus::Encodable, Block, BlockHash,
    BlockHeader, OutPoint, Transaction, TxIn, TxOut, Witness,
  },
  core::str::FromStr,
  electrsd::bitcoind::{
    bitcoincore_rpc::{
      bitcoin::{Address, Amount},
      RpcApi,
    },
    BitcoinD as Bitcoind,
  },
  executable_path::executable_path,
  nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
  },
  regex::Regex,
  std::{
    collections::BTreeMap,
    error::Error,
    ffi::OsString,
    fs,
    net::TcpListener,
    process::{Command, Stdio},
    str,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::{Duration, Instant},
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
mod nft;
mod range;
mod rpc_server;
mod server;
mod supply;
mod traits;
mod version;
mod wallet;

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

enum Expected {
  String(String),
  Regex(Regex),
  Ignore,
}

enum Event {
  Block(Block),
  Request(String, u16, String),
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
  bitcoind: Bitcoind,
  envs: Vec<(OsString, OsString)>,
  events: Vec<Event>,
  expected_status: i32,
  expected_stderr: Expected,
  expected_stdout: Expected,
  tempdir: TempDir,
}

impl Test {
  fn new() -> Result<Self> {
    Ok(Self::with_tempdir(TempDir::new()?)?)
  }

  fn with_tempdir(tempdir: TempDir) -> Result<Self> {
    let mut config = electrsd::bitcoind::Conf::default();

    config.args = vec!["-regtest", "-fallbackfee=0.0001", "-wallet"];

    Ok(Self {
      args: Vec::new(),
      bitcoind: Bitcoind::with_conf(electrsd::bitcoind::downloaded_exe_path()?, &config)?,
      envs: Vec::new(),
      events: Vec::new(),
      expected_status: 0,
      expected_stderr: Expected::Ignore,
      expected_stdout: Expected::String(String::new()),
      tempdir,
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
      expected_stdout: Expected::String(expected_stdout.as_ref().to_owned()),
      ..self
    }
  }

  fn stdout_regex(self, expected_stdout: impl AsRef<str>) -> Self {
    Self {
      expected_stdout: Expected::Regex(
        Regex::new(&format!("(?s)^{}$", expected_stdout.as_ref())).unwrap(),
      ),
      ..self
    }
  }

  fn set_home_to_tempdir(mut self) -> Self {
    self
      .envs
      .push((OsString::from("HOME"), OsString::from(self.tempdir.path())));

    self
  }

  fn expected_stderr(self, expected_stderr: &str) -> Self {
    Self {
      expected_stderr: Expected::String(expected_stderr.to_owned()),
      ..self
    }
  }

  fn stderr_regex(self, expected_stderr: impl AsRef<str>) -> Self {
    Self {
      expected_stderr: Expected::Regex(
        Regex::new(&format!("(?s)^{}$", expected_stderr.as_ref())).unwrap(),
      ),
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

  fn request(mut self, path: &str, status: u16, response: &str) -> Self {
    self.events.push(Event::Request(
      path.to_string(),
      status,
      response.to_string(),
    ));
    self
  }

  fn run(self) -> Result {
    self.test(None).map(|_| ())
  }

  fn output(self) -> Result<Output> {
    self.test(None)
  }

  fn run_server(self, port: u16) -> Result {
    self.test(Some(port)).map(|_| ())
  }

  fn blocks(&self) -> impl Iterator<Item = &Block> + '_ {
    self.events.iter().filter_map(|event| match event {
      Event::Block(block) => Some(block),
      _ => None,
    })
  }

  fn test(self, port: Option<u16>) -> Result<Output> {
    for (b, block) in self.blocks().enumerate() {
      for (t, transaction) in block.txdata.iter().enumerate() {
        eprintln!("{b}.{t}: {}", transaction.txid());
      }
    }

    let (blocks, close_handle, calls, rpc_server_port) = if port.is_some() {
      RpcServer::spawn(Vec::new())
    } else {
      RpcServer::spawn(self.blocks().cloned().collect())
    };

    let child = Command::new(executable_path("ord"))
      .envs(self.envs)
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(if !matches!(self.expected_stderr, Expected::Ignore) {
        Stdio::piped()
      } else {
        Stdio::inherit()
      })
      .current_dir(&self.tempdir)
      .arg(format!(
        "--rpc-url={}",
        self.bitcoind.params.rpc_socket.to_string()
      ))
      .arg(format!(
        "--cookie-file={}",
        self.bitcoind.params.cookie_file.display()
      ))
      .args(self.args)
      .spawn()?;

    let mut successful_requests = 0;

    if let Some(port) = port {
      let client = reqwest::blocking::Client::new();

      let start = Instant::now();
      let mut healthy = false;

      loop {
        if let Ok(response) = client
          .get(&format!("http://127.0.0.1:{port}/status"))
          .send()
        {
          if response.status().is_success() {
            healthy = true;
            break;
          }
        }

        if Instant::now() - start > Duration::from_secs(1) {
          break;
        }

        sleep(Duration::from_millis(100));
      }

      if healthy {
        for event in &self.events {
          match event {
            Event::Block(block) => {
              blocks.lock().unwrap().push(block.clone());
              thread::sleep(Duration::from_millis(200));
            }
            Event::Request(request, status, expected_response) => {
              let response = client
                .get(&format!("http://127.0.0.1:{port}/{request}"))
                .send()?;
              assert_eq!(response.status().as_u16(), *status);
              assert_eq!(response.text()?, *expected_response);
              successful_requests += 1;
            }
          }
        }
      }

      signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT)?;
    }

    let output = child.wait_with_output()?;

    close_handle.close();

    let stdout = str::from_utf8(&output.stdout)?;
    let stderr = str::from_utf8(&output.stderr)?;

    if output.status.code() != Some(self.expected_status) {
      panic!(
        "Test failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status, stdout, stderr
      );
    }

    let log_line_re = Regex::new(r"(?m)^\[.*\n")?;

    for log_line in log_line_re.find_iter(stderr) {
      print!("{}", log_line.as_str())
    }

    let stripped_stderr = log_line_re.replace_all(stderr, "");

    match self.expected_stderr {
      Expected::String(expected_stderr) => assert_eq!(stripped_stderr, expected_stderr),
      Expected::Regex(expected_stderr) => assert!(
        expected_stderr.is_match(&stripped_stderr),
        "stderr did not match regex: {:?}",
        stripped_stderr
      ),
      Expected::Ignore => {}
    }

    match self.expected_stdout {
      Expected::String(expected_stdout) => assert_eq!(stdout, expected_stdout),
      Expected::Regex(expected_stdout) => assert!(
        expected_stdout.is_match(stdout),
        "stdout did not match regex: {:?}",
        stdout
      ),
      Expected::Ignore => {}
    }

    assert_eq!(
      successful_requests,
      self
        .events
        .iter()
        .filter(|event| matches!(event, Event::Request(..)))
        .count(),
      "Unsuccessful requests"
    );

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
    // basically just need to replace this `Block` construction
    // with calls to `bitcoincore_rpc` api

    // 1. Create block
    // 2. Create transaction

    let core_address = self.bitcoind.client.get_new_address(None, None).unwrap();

    let block_hashes = self
      .bitcoind
      .client
      .generate_to_address(101, &core_address)
      .unwrap();

    let mut block = self
      .bitcoind
      .client
      .get_block(block_hashes.first().unwrap())
      .unwrap();

    block.header = BlockHeader {
      version: 0,
      prev_blockhash: self
        .blocks()
        .last()
        .map(Block::block_hash)
        .unwrap_or_default(),
      merkle_root: Default::default(),
      time: 0,
      bits: 0,
      nonce: 0,
    };

    block.txdata = if coinbase.include_coinbase_transaction {
      vec![Transaction {
        version: 0,
        lock_time: 0,
        input: vec![TxIn {
          previous_output: OutPoint::null(),
          script_sig: if coinbase.include_height {
            script::Builder::new()
              .push_scriptint(self.blocks().count().try_into().unwrap())
              .into_script()
          } else {
            script::Builder::new().into_script()
          },
          sequence: 0,
          witness: Witness::new(),
        }],
        output: vec![TxOut {
          value: coinbase.subsidy,
          script_pubkey: script::Builder::new().into_script(),
        }],
      }]
    } else {
      Vec::new()
    };

    // self.events.push(Event::Block(Block {
    //   header: BlockHeader {
    //     version: 0,
    //     prev_blockhash: self
    //       .blocks()
    //       .last()
    //       .map(Block::block_hash)
    //       .unwrap_or_default(),
    //     merkle_root: Default::default(),
    //     time: 0,
    //     bits: 0,
    //     nonce: 0,
    //   },
    //   txdata: if coinbase.include_coinbase_transaction {
    //     vec![Transaction {
    //       version: 0,
    //       lock_time: 0,
    //       input: vec![TxIn {
    //         previous_output: OutPoint::null(),
    //         script_sig: if coinbase.include_height {
    //           script::Builder::new()
    //             .push_scriptint(self.blocks().count().try_into().unwrap())
    //             .into_script()
    //         } else {
    //           script::Builder::new().into_script()
    //         },
    //         sequence: 0,
    //         witness: Witness::new(),
    //       }],
    //       output: vec![TxOut {
    //         value: coinbase.subsidy,
    //         script_pubkey: script::Builder::new().into_script(),
    //       }],
    //     }]
    //   } else {
    //     Vec::new()
    //   },
    // }));

    self.events.push(Event::Block(block));

    self
  }

  fn transaction(mut self, options: TransactionOptions) -> Self {
    let input_value = options
      .slots
      .iter()
      .map(|slot| self.blocks().nth(slot.0).unwrap().txdata[slot.1].output[slot.2].value)
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
            txid: self.blocks().nth(slot.0).unwrap().txdata[slot.1].txid(),
            vout: slot.2 as u32,
          },
          script_sig: script::Builder::new().into_script(),
          sequence: 0,
          witness: Witness::new(),
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

    let block = self
      .events
      .iter_mut()
      .rev()
      .find_map(|event| match event {
        Event::Block(block) => Some(block),
        _ => None,
      })
      .unwrap();

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

  fn write(self, path: &str, contents: &str) -> Result<Self> {
    fs::write(self.tempdir.path().join(path), contents)?;
    Ok(self)
  }
}
