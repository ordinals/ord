use super::*;

pub(crate) struct SlowTest {
  args: Vec<String>,
  expected_status: i32,
  expected_stderr: Expected,
  expected_stdout: Expected,
  state: State,
}

impl SlowTest {
  pub(crate) fn new() -> Self {
    Self::with_state(State::new())
  }

  pub(crate) fn with_state(state: State) -> Self {
    let test = Self {
      args: Vec::new(),
      state,
      expected_status: 0,
      expected_stderr: Expected::Ignore,
      expected_stdout: Expected::String(String::new()),
    };

    test.state.sync();

    test
  }

  pub(crate) fn command(self, args: &str) -> Self {
    Self {
      args: args.split_whitespace().map(str::to_owned).collect(),
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

  pub(crate) fn run(self) {
    self.output();
  }

  pub(crate) fn output(self) -> State {
    let output = Command::new(executable_path("ord"))
      .env("HOME", self.state.tempdir.path())
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(if !matches!(self.expected_stderr, Expected::Ignore) {
        Stdio::piped()
      } else {
        Stdio::inherit()
      })
      .current_dir(&self.state.tempdir)
      .arg(format!(
        "--rpc-url=localhost:{}",
        self.state.bitcoind_rpc_port
      ))
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

    self.state
  }

  pub(crate) fn blocks(self, n: u64) -> Self {
    self.state.blocks(n);
    self
  }

  pub(crate) fn transaction(self, options: TransactionOptions) -> Self {
    self.state.transaction(options);
    self
  }
}
