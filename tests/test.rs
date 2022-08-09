use super::*;

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

pub(crate) struct Output {
  pub(crate) stdout: String,
  pub(crate) state: State,
}

pub(crate) struct Test {
  args: Vec<String>,
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
    self.output();
  }

  pub(crate) fn output(self) -> Output {
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

    Output {
      stdout: stdout.to_string(),
      state: self.state,
    }
  }

  pub(crate) fn blocks(self, n: u64) -> Self {
    self.state.blocks(n);
    self
  }

  pub(crate) fn transaction(self, options: TransactionOptions) -> Self {
    self.state.transaction(options);
    self
  }

  pub(crate) fn write(self, path: &str, contents: &str) -> Self {
    fs::write(self.state.tempdir.path().join(path), contents).unwrap();
    self
  }
}
