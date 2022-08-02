#![allow(clippy::type_complexity)]

use {
  bdk::{
    blockchain::{
      rpc::{RpcBlockchain, RpcConfig},
      ConfigurableBlockchain,
    },
    database::MemoryDatabase,
    keys::bip39::Mnemonic,
    template::Bip84,
    wallet::{signer::SignOptions, AddressIndex, SyncOptions, Wallet},
    KeychainKind,
  },
  bitcoin::hash_types::Txid,
  bitcoin::{network::constants::Network, Block, OutPoint},
  bitcoincore_rpc::{Client, RawTx, RpcApi},
  executable_path::executable_path,
  log::LevelFilter,
  nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
  },
  regex::Regex,
  std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    net::TcpListener,
    process::{Child, Command, Stdio},
    str,
    sync::Once,
    thread::sleep,
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
mod server;
mod supply;
mod traits;
mod version;
mod wallet;

static ONCE: Once = Once::new();

fn free_port() -> u16 {
  TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port()
}

#[derive(Debug)]
enum Expected {
  String(String),
  Regex(Regex),
  Ignore,
}

impl Expected {
  fn regex(pattern: &str) -> Self {
    Self::Regex(Regex::new(&format!("^(?s){}$", pattern)).unwrap())
  }

  fn assert_match(&self, output: &str) {
    match self {
      Self::String(string) => assert_eq!(output, string),
      Self::Regex(regex) => assert!(
        regex.is_match(output),
        "output did not match regex: {:?}",
        output
      ),
      Self::Ignore => {}
    }
  }
}

enum Event<'a> {
  Blocks(u64),
  Request(String, u16, String),
  Transaction(TransactionOptions<'a>),
}

struct Output {
  stdout: String,
  state: State,
}

struct TransactionOptions<'a> {
  slots: &'a [(usize, usize, usize)],
  output_count: usize,
  fee: u64,
}

struct Test<'a> {
  args: Vec<String>,
  envs: Vec<(OsString, OsString)>,
  events: Vec<Event<'a>>,
  expected_status: i32,
  expected_stderr: Expected,
  expected_stdout: Expected,
  state: State,
}

struct State {
  bitcoind: Child,
  tempdir: TempDir,
  client: Client,
  wallet: Wallet<MemoryDatabase>,
  blockchain: RpcBlockchain,
  rpc_port: u16,
}

impl State {
  fn new() -> Self {
    let tempdir = TempDir::new().unwrap();

    fs::create_dir(tempdir.path().join("bitcoin")).unwrap();

    let rpc_port = free_port();

    let bitcoind = Command::new("bitcoind")
      .stdout(if log::max_level() >= LevelFilter::Info {
        Stdio::inherit()
      } else {
        Stdio::piped()
      })
      .args(&[
        "-minrelaytxfee=0",
        "-blockmintxfee=0",
        "-dustrelayfee=0",
        "-maxtxfee=21000000",
        "-datadir=bitcoin",
        "-regtest",
        "-networkactive=0",
        "-listen=0",
        &format!("-rpcport={rpc_port}"),
      ])
      .current_dir(&tempdir.path())
      .spawn()
      .unwrap();

    let cookiefile = tempdir.path().join("bitcoin/regtest/.cookie");

    while !cookiefile.is_file() {}

    let client = Client::new(
      &format!("localhost:{rpc_port}"),
      bitcoincore_rpc::Auth::CookieFile(cookiefile.clone()),
    )
    .unwrap();

    for attempt in 0..=300 {
      match client.get_blockchain_info() {
        Ok(_) => break,
        Err(err) => {
          if attempt == 300 {
            panic!("Failed to connect to bitcoind: {err}");
          }
        }
      }
      sleep(Duration::from_millis(100));
    }

    let wallet = Wallet::new(
      Bip84(
        (
          Mnemonic::parse("book fit fly ketchup also elevator scout mind edit fatal where rookie")
            .unwrap(),
          None,
        ),
        KeychainKind::External,
      ),
      None,
      Network::Regtest,
      MemoryDatabase::new(),
    )
    .unwrap();

    let blockchain = RpcBlockchain::from_config(&RpcConfig {
      url: format!("localhost:{rpc_port}"),
      auth: bdk::blockchain::rpc::Auth::Cookie { file: cookiefile },
      network: Network::Regtest,
      wallet_name: "test".to_string(),
      skip_blocks: None,
    })
    .unwrap();

    State {
      tempdir,
      rpc_port,
      bitcoind,
      client,
      wallet,
      blockchain,
    }
  }
}

impl Drop for State {
  fn drop(&mut self) {
    self.bitcoind.kill().unwrap();
  }
}

impl<'a> Test<'a> {
  fn new() -> Self {
    Self::with_state(State::new())
  }

  fn with_state(state: State) -> Self {
    ONCE.call_once(|| {
      env_logger::init();
    });

    let test = Self {
      args: Vec::new(),
      state,
      envs: Vec::new(),
      events: Vec::new(),
      expected_status: 0,
      expected_stderr: Expected::Ignore,
      expected_stdout: Expected::String(String::new()),
    };

    test.sync();

    test
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
      expected_stdout: Expected::regex(expected_stdout.as_ref()),
      ..self
    }
  }

  // TODO: do this always
  fn set_home_to_tempdir(mut self) -> Self {
    self.envs.push((
      OsString::from("HOME"),
      OsString::from(self.state.tempdir.path()),
    ));

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
      expected_stderr: Expected::regex(expected_stderr.as_ref()),
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

  fn run(self) {
    self.test(None);
  }

  fn output(self) -> Output {
    self.test(None)
  }

  fn run_server(self, port: u16) {
    self.test(Some(port));
  }

  fn get_block(&self, height: u64) -> Block {
    self
      .state
      .client
      .get_block(&self.state.client.get_block_hash(height).unwrap())
      .unwrap()
  }

  fn run_server_output(self, port: u16) -> Output {
    self.test(Some(port))
  }

  fn sync(&self) {
    self
      .state
      .wallet
      .sync(&self.state.blockchain, SyncOptions::default())
      .unwrap();
  }

  fn test(self, port: Option<u16>) -> Output {
    let client = reqwest::blocking::Client::new();

    log::info!("Spawning child process...");

    let (healthy, child) = if let Some(port) = port {
      let child = Command::new(executable_path("ord"))
        .envs(self.envs.clone())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(if !matches!(self.expected_stderr, Expected::Ignore) {
          Stdio::piped()
        } else {
          Stdio::inherit()
        })
        .current_dir(&self.state.tempdir)
        .arg(format!("--rpc-url=localhost:{}", self.state.rpc_port))
        .arg("--cookie-file=bitcoin/regtest/.cookie")
        .args(self.args.clone())
        .spawn()
        .unwrap();

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

      (healthy, Some(child))
    } else {
      (false, None)
    };

    log::info!(
      "Server status: {}",
      if healthy { "healthy" } else { "not healthy " }
    );

    let mut successful_requests = 0;

    for event in &self.events {
      match event {
        Event::Blocks(n) => {
          self
            .state
            .client
            .generate_to_address(
              *n,
              &self
                .state
                .wallet
                .get_address(AddressIndex::Peek(0))
                .unwrap()
                .address,
            )
            .unwrap();
        }
        Event::Request(request, status, expected_response) => {
          if healthy {
            let response = client
              .get(&format!("http://127.0.0.1:{}/{request}", port.unwrap()))
              .send()
              .unwrap();
            log::info!("{:?}", response);
            assert_eq!(response.status().as_u16(), *status);
            assert_eq!(response.text().unwrap(), *expected_response);
            successful_requests += 1;
          } else {
            panic!("Tried to make a request when unhealthy");
          }
        }
        Event::Transaction(options) => {
          self.sync();

          let input_value = options
            .slots
            .iter()
            .map(|slot| self.get_block(slot.0 as u64).txdata[slot.1].output[slot.2].value)
            .sum::<u64>();

          let output_value = input_value - options.fee;

          let (mut psbt, _) = {
            let mut builder = self.state.wallet.build_tx();

            builder
              .manually_selected_only()
              .fee_absolute(options.fee)
              .allow_dust(true)
              .add_utxos(
                &options
                  .slots
                  .iter()
                  .map(|slot| OutPoint {
                    txid: self.get_block(slot.0 as u64).txdata[slot.1].txid(),
                    vout: slot.2 as u32,
                  })
                  .collect::<Vec<OutPoint>>(),
              )
              .unwrap()
              .set_recipients(vec![
                (
                  self
                    .state
                    .wallet
                    .get_address(AddressIndex::Peek(0))
                    .unwrap()
                    .address
                    .script_pubkey(),
                  output_value / options.output_count as u64
                );
                options.output_count
              ]);

            builder.finish().unwrap()
          };

          if !self
            .state
            .wallet
            .sign(&mut psbt, SignOptions::default())
            .unwrap()
          {
            panic!("Failed to sign transaction");
          }

          self
            .state
            .client
            .call::<Txid>(
              "sendrawtransaction",
              &[psbt.extract_tx().raw_hex().into(), 21000000.into()],
            )
            .unwrap();
        }
      }
    }

    let child = if let Some(child) = child {
      signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT).unwrap();
      child
    } else {
      Command::new(executable_path("ord"))
        .envs(self.envs.clone())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(if !matches!(self.expected_stderr, Expected::Ignore) {
          Stdio::piped()
        } else {
          Stdio::inherit()
        })
        .current_dir(&self.state.tempdir)
        .arg(format!("--rpc-url=localhost:{}", self.state.rpc_port))
        .arg("--cookie-file=bitcoin/regtest/.cookie")
        .args(self.args.clone())
        .spawn()
        .unwrap()
    };

    let output = child.wait_with_output().unwrap();

    let stdout = str::from_utf8(&output.stdout).unwrap();
    let stderr = str::from_utf8(&output.stderr).unwrap();

    if output.status.code() != Some(self.expected_status) {
      panic!(
        "Test failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status, stdout, stderr
      );
    }

    let log_line_re = Regex::new(r"(?m)^\[.*\n").unwrap();

    for log_line in log_line_re.find_iter(stderr) {
      print!("{}", log_line.as_str())
    }

    let stripped_stderr = log_line_re.replace_all(stderr, "");

    self.expected_stderr.assert_match(&stripped_stderr);
    self.expected_stdout.assert_match(stdout);

    assert_eq!(
      successful_requests,
      self
        .events
        .iter()
        .filter(|event| matches!(event, Event::Request(..)))
        .count(),
      "Unsuccessful requests"
    );

    Output {
      stdout: stdout.to_string(),
      state: self.state,
    }
  }

  fn blocks(mut self, n: u64) -> Self {
    self.events.push(Event::Blocks(n));
    self
  }

  fn transaction(mut self, options: TransactionOptions<'a>) -> Self {
    self.events.push(Event::Transaction(options));
    self
  }

  fn write(self, path: &str, contents: &str) -> Self {
    fs::write(self.state.tempdir.path().join(path), contents).unwrap();
    self
  }
}
