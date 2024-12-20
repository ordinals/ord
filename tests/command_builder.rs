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

pub(crate) struct Spawn {
  pub(crate) child: Child,
  expected_exit_code: i32,
  expected_stderr: Expected,
  expected_stdout: Expected,
  tempdir: Arc<TempDir>,
}

impl Spawn {
  #[track_caller]
  fn run(self) -> (Arc<TempDir>, String) {
    let output = self.child.wait_with_output().unwrap();

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

    (self.tempdir, stdout.into())
  }

  #[track_caller]
  pub(crate) fn run_and_deserialize_output<T: DeserializeOwned>(self) -> T {
    let stdout = self.stdout_regex(".*").run_and_extract_stdout();
    serde_json::from_str(&stdout)
      .unwrap_or_else(|err| panic!("Failed to deserialize JSON: {err}\n{stdout}"))
  }

  #[track_caller]
  pub(crate) fn run_and_extract_stdout(self) -> String {
    self.run().1
  }

  pub(crate) fn stdout_regex(self, expected_stdout: impl AsRef<str>) -> Self {
    Self {
      expected_stdout: Expected::regex(expected_stdout.as_ref()),
      ..self
    }
  }
}

pub(crate) struct CommandBuilder {
  args: Vec<String>,
  core_cookie_file: Option<PathBuf>,
  core_url: Option<String>,
  env: BTreeMap<String, OsString>,
  expected_exit_code: i32,
  expected_stderr: Expected,
  expected_stdout: Expected,
  integration_test: bool,
  ord_url: Option<Url>,
  stderr: bool,
  stdin: Vec<u8>,
  stdout: bool,
  tempdir: Arc<TempDir>,
}

impl CommandBuilder {
  pub(crate) fn new(args: impl ToArgs) -> Self {
    Self {
      args: args.to_args(),
      core_cookie_file: None,
      core_url: None,
      env: BTreeMap::new(),
      expected_exit_code: 0,
      expected_stderr: Expected::String(String::new()),
      expected_stdout: Expected::String(String::new()),
      integration_test: true,
      ord_url: None,
      stderr: true,
      stdin: Vec::new(),
      stdout: true,
      tempdir: Arc::new(TempDir::new().unwrap()),
    }
  }

  pub(crate) fn env(mut self, key: &str, value: impl AsRef<OsStr>) -> Self {
    self.env.insert(key.into(), value.as_ref().into());
    self
  }

  pub(crate) fn integration_test(self, integration_test: bool) -> Self {
    Self {
      integration_test,
      ..self
    }
  }

  pub(crate) fn write(self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Self {
    fs::write(self.tempdir.path().join(path), contents).unwrap();
    self
  }

  pub(crate) fn core(self, core: &mockcore::Handle) -> Self {
    Self {
      core_url: Some(core.url()),
      core_cookie_file: Some(core.cookie_file()),
      ..self
    }
  }

  pub(crate) fn ord(self, ord: &TestServer) -> Self {
    Self {
      ord_url: Some(ord.url()),
      ..self
    }
  }

  #[allow(unused)]
  pub(crate) fn stderr(self, stderr: bool) -> Self {
    Self { stderr, ..self }
  }

  pub(crate) fn stdin(self, stdin: Vec<u8>) -> Self {
    Self { stdin, ..self }
  }

  #[allow(unused)]
  pub(crate) fn stdout(self, stdout: bool) -> Self {
    Self { stdout, ..self }
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

    if let Some(rpc_server_url) = &self.core_url {
      command.args([
        "--bitcoin-rpc-url",
        rpc_server_url,
        "--cookie-file",
        self.core_cookie_file.as_ref().unwrap().to_str().unwrap(),
      ]);
    }

    let mut args = Vec::new();

    for arg in self.args.iter() {
      args.push(arg.clone());
      if arg == "wallet" {
        if let Some(ord_server_url) = &self.ord_url {
          args.push("--server-url".to_string());
          args.push(ord_server_url.to_string());
        }
      }
    }

    for (key, value) in &self.env {
      command.env(key, value);
    }

    if self.integration_test {
      command.env("ORD_INTEGRATION_TEST", "1");
    }

    command
      .stdin(Stdio::piped())
      .stdout(if self.stdout {
        Stdio::piped()
      } else {
        Stdio::inherit()
      })
      .stderr(if self.stderr {
        Stdio::piped()
      } else {
        Stdio::inherit()
      })
      .current_dir(&*self.tempdir)
      .arg("--datadir")
      .arg(self.tempdir.path())
      .args(&args);

    command
  }

  #[track_caller]
  pub(crate) fn spawn(self) -> Spawn {
    let mut command = self.command();
    let child = command.spawn().unwrap();

    child
      .stdin
      .as_ref()
      .unwrap()
      .write_all(&self.stdin)
      .unwrap();

    Spawn {
      child,
      expected_exit_code: self.expected_exit_code,
      expected_stderr: self.expected_stderr,
      expected_stdout: self.expected_stdout,
      tempdir: self.tempdir,
    }
  }

  #[track_caller]
  pub(crate) fn run(self) -> (Arc<TempDir>, String) {
    self.spawn().run()
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
    match serde_json::from_str(&stdout) {
      Ok(output) => output,
      Err(err) => panic!("Failed to deserialize JSON: {err}\n{stdout}"),
    }
  }
}
