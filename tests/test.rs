use super::*;

pub(crate) struct Test {
  args: Vec<String>,
  envs: Vec<(OsString, OsString)>,
  expected_status: i32,
  expected_stderr: Expected,
  expected_stdout: Expected,
  state: State,
}

impl Test {
  pub(crate) fn new() -> Self {
    Self::with_state(State::new())
  }

  pub(crate) fn with_state(state: State) -> Self {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
      env_logger::init();
    });

    let test = Self {
      args: Vec::new(),
      state,
      envs: Vec::new(),
      expected_status: 0,
      expected_stderr: Expected::Ignore,
      expected_stdout: Expected::String(String::new()),
    };

    test.sync();

    test
  }

  pub(crate) fn command(self, args: &str) -> Self {
    Self {
      args: args.split_whitespace().map(str::to_owned).collect(),
      ..self
    }
  }

  pub(crate) fn args(self, args: &[&str]) -> Self {
    Self {
      args: self
        .args
        .into_iter()
        .chain(args.iter().cloned().map(str::to_owned))
        .collect(),
      ..self
    }
  }

  pub(crate) fn expected_stdout(self, expected_stdout: impl AsRef<str>) -> Self {
    Self {
      expected_stdout: Expected::String(expected_stdout.as_ref().to_owned()),
      ..self
    }
  }

  pub(crate) fn stdout_regex(self, expected_stdout: impl AsRef<str>) -> Self {
    Self {
      expected_stdout: Expected::regex(expected_stdout.as_ref()),
      ..self
    }
  }

  // TODO: do this always
  pub(crate) fn set_home_to_tempdir(mut self) -> Self {
    self.envs.push((
      OsString::from("HOME"),
      OsString::from(self.state.tempdir.path()),
    ));

    self
  }

  pub(crate) fn expected_stderr(self, expected_stderr: &str) -> Self {
    Self {
      expected_stderr: Expected::String(expected_stderr.to_owned()),
      ..self
    }
  }

  pub(crate) fn stderr_regex(self, expected_stderr: impl AsRef<str>) -> Self {
    Self {
      expected_stderr: Expected::regex(expected_stderr.as_ref()),
      ..self
    }
  }

  pub(crate) fn expected_status(self, expected_status: i32) -> Self {
    Self {
      expected_status,
      ..self
    }
  }

  pub(crate) fn ignore_stdout(self) -> Self {
    Self {
      expected_stdout: Expected::Ignore,
      ..self
    }
  }

  pub(crate) fn run(self) {
    self.test(None);
  }

  pub(crate) fn output(self) -> Output {
    self.test(None)
  }

  pub(crate) fn run_server(self, port: u16) {
    self.test(Some(port));
  }

  fn get_block(&self, height: u64) -> Block {
    self
      .state
      .client
      .get_block(&self.state.client.get_block_hash(height).unwrap())
      .unwrap()
  }

  pub(crate) fn run_server_output(self, port: u16) -> Output {
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
    let output = Command::new(executable_path("ord"))
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
      .output()
      .unwrap();

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

    Output {
      stdout: stdout.to_string(),
      state: self.state,
    }
  }

  pub(crate) fn blocks(mut self, n: u64) -> Self {
    self
      .state
      .client
      .generate_to_address(
        n,
        &self
          .state
          .wallet
          .get_address(AddressIndex::Peek(0))
          .unwrap()
          .address,
      )
      .unwrap();

    self
  }

  pub(crate) fn transaction(mut self, options: TransactionOptions) -> Self {
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
    self
  }

  pub(crate) fn write(self, path: &str, contents: &str) -> Self {
    fs::write(self.state.tempdir.path().join(path), contents).unwrap();
    self
  }

  pub(crate) fn request(mut self, path: &str, status: u16, expected_response: &str) -> Self {
    panic!()
  }
}
