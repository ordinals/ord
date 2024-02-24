use super::*;

pub(crate) trait ToArgs {
  fn to_args(&self) -> Vec<String>;
}

impl ToArgs for String {
  fn to_args(&self) -> Vec<String> {
    self.as_str().to_args()
  }
}

impl ToArgs for &str {
  fn to_args(&self) -> Vec<String> {
    self.split_whitespace().map(str::to_string).collect()
  }
}

impl<const N: usize> ToArgs for [&str; N] {
  fn to_args(&self) -> Vec<String> {
    self.iter().cloned().map(str::to_string).collect()
  }
}

impl ToArgs for Vec<String> {
  fn to_args(&self) -> Vec<String> {
    self.clone()
  }
}

pub(crate) struct CommandBuilder {
  args: Vec<String>,
  bitcoin_rpc_server_cookie_file: Option<PathBuf>,
  bitcoin_rpc_server_url: Option<String>,
  env: BTreeMap<String, String>,
  expected_exit_code: i32,
  expected_stderr: Expected,
  expected_stdout: Expected,
  ord_rpc_server_url: Option<Url>,
  stdin: Vec<u8>,
  tempdir: Arc<TempDir>,
}

impl CommandBuilder {
  pub(crate) fn new(args: impl ToArgs) -> Self {
    Self {
      args: args.to_args(),
      env: BTreeMap::new(),
      expected_exit_code: 0,
      expected_stderr: Expected::String(String::new()),
      expected_stdout: Expected::String(String::new()),
      ord_rpc_server_url: None,
      bitcoin_rpc_server_cookie_file: None,
      bitcoin_rpc_server_url: None,
      stdin: Vec::new(),
      tempdir: Arc::new(TempDir::new().unwrap()),
    }
  }

  pub(crate) fn env(mut self, key: &str, value: &str) -> Self {
    self.env.insert(key.into(), value.into());
    self
  }

  pub(crate) fn write(self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Self {
    fs::write(self.tempdir.path().join(path), contents).unwrap();
    self
  }

  pub(crate) fn bitcoin_rpc_server(
    self,
    bitcoin_rpc_server: &test_bitcoincore_rpc::Handle,
  ) -> Self {
    Self {
      bitcoin_rpc_server_url: Some(bitcoin_rpc_server.url()),
      bitcoin_rpc_server_cookie_file: Some(bitcoin_rpc_server.cookie_file()),
      ..self
    }
  }

  pub(crate) fn ord_rpc_server(self, ord_rpc_server: &TestServer) -> Self {
    Self {
      ord_rpc_server_url: Some(ord_rpc_server.url()),
      ..self
    }
  }

  pub(crate) fn stdin(self, stdin: Vec<u8>) -> Self {
    Self { stdin, ..self }
  }

  pub(crate) fn stdout_regex(self, expected_stdout: impl AsRef<str>) -> Self {
    Self {
      expected_stdout: Expected::regex(expected_stdout.as_ref()),
      ..self
    }
  }

  pub(crate) fn expected_stderr(self, expected_stderr: impl AsRef<str>) -> Self {
    Self {
      expected_stderr: Expected::String(expected_stderr.as_ref().to_owned()),
      ..self
    }
  }

  pub(crate) fn stderr_regex(self, expected_stderr: impl AsRef<str>) -> Self {
    Self {
      expected_stderr: Expected::regex(expected_stderr.as_ref()),
      ..self
    }
  }

  pub(crate) fn expected_exit_code(self, expected_exit_code: i32) -> Self {
    Self {
      expected_exit_code,
      ..self
    }
  }

  pub(crate) fn temp_dir(self, tempdir: Arc<TempDir>) -> Self {
    Self { tempdir, ..self }
  }

  pub(crate) fn command(&self) -> Command {
    let mut command = Command::new(executable_path("ord"));

    if let Some(rpc_server_url) = &self.bitcoin_rpc_server_url {
      command.args([
        "--rpc-url",
        rpc_server_url,
        "--cookie-file",
        self
          .bitcoin_rpc_server_cookie_file
          .as_ref()
          .unwrap()
          .to_str()
          .unwrap(),
      ]);
    }

    let mut args = Vec::new();

    for arg in self.args.iter() {
      args.push(arg.clone());
      if arg == "wallet" {
        if let Some(ord_server_url) = &self.ord_rpc_server_url {
          args.push("--server-url".to_string());
          args.push(ord_server_url.to_string());
        }
      }
    }

    for (key, value) in &self.env {
      command.env(key, value);
    }

    command
      .env("ORD_INTEGRATION_TEST", "1")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .current_dir(&*self.tempdir)
      .arg("--data-dir")
      .arg(self.tempdir.path())
      .args(&args);

    command
  }

  #[track_caller]
  fn run(self) -> (TempDir, String) {
    let mut command = self.command();
    let child = command.spawn().unwrap();

    child
      .stdin
      .as_ref()
      .unwrap()
      .write_all(&self.stdin)
      .unwrap();

    let output = child.wait_with_output().unwrap();

    let stdout = str::from_utf8(&output.stdout).unwrap();
    let stderr = str::from_utf8(&output.stderr).unwrap();
    if output.status.code() != Some(self.expected_exit_code) {
      panic!(
        "Test failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status, stdout, stderr
      );
    }

    self.expected_stderr.assert_match(stderr);
    self.expected_stdout.assert_match(stdout);

    (Arc::try_unwrap(self.tempdir).unwrap(), stdout.into())
  }

  pub(crate) fn run_and_extract_file(self, path: impl AsRef<Path>) -> String {
    let tempdir = self.run().0;
    fs::read_to_string(tempdir.path().join(path)).unwrap()
  }

  #[track_caller]
  pub(crate) fn run_and_extract_stdout(self) -> String {
    self.run().1
  }

  #[track_caller]
  pub(crate) fn run_and_deserialize_output<T: DeserializeOwned>(self) -> T {
    let stdout = self.stdout_regex(".*").run_and_extract_stdout();
    serde_json::from_str(&stdout)
      .unwrap_or_else(|err| panic!("Failed to deserialize JSON: {err}\n{stdout}"))
  }
}
