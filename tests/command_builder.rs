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
  expected_exit_code: i32,
  expected_stderr: Expected,
  expected_stdout: Expected,
  rpc_server_url: Option<String>,
  tempdir: TempDir,
}

impl CommandBuilder {
  pub(crate) fn new(args: impl ToArgs) -> Self {
    Self {
      args: args.to_args(),
      expected_exit_code: 0,
      expected_stderr: Expected::String(String::new()),
      expected_stdout: Expected::String(String::new()),
      rpc_server_url: None,
      tempdir: TempDir::new().unwrap(),
    }
  }

  pub(crate) fn write(self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Self {
    fs::write(self.tempdir.path().join(path), contents).unwrap();
    self
  }

  pub(crate) fn rpc_server(self, rpc_server: &test_bitcoincore_rpc::Handle) -> Self {
    Self {
      rpc_server_url: Some(rpc_server.url()),
      ..self
    }
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

  pub(crate) fn temp_dir(self, tempdir: TempDir) -> Self {
    Self { tempdir, ..self }
  }

  pub(crate) fn command(&self) -> Command {
    let mut command = Command::new(executable_path("ord"));

    if let Some(rpc_server_url) = &self.rpc_server_url {
      let cookiefile = self.tempdir.path().join("cookie");
      fs::write(&cookiefile, "username:password").unwrap();
      command.args([
        "--rpc-url",
        rpc_server_url,
        "--cookie-file",
        cookiefile.to_str().unwrap(),
      ]);
    }

    command
      .env("ORD_INTEGRATION_TEST", "1")
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .current_dir(&self.tempdir)
      .arg("--data-dir")
      .arg(self.tempdir.path())
      .args(&self.args);

    command
  }

  pub(crate) fn run_and_extract_file(self, path: impl AsRef<Path>) -> String {
    let output = self.command().output().unwrap();
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

    fs::read_to_string(self.tempdir.path().join(path)).unwrap()
  }

  pub(crate) fn run_and_extract_stdout(self) -> String {
    let output = self.command().output().unwrap();
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

    stdout.into()
  }

  pub(crate) fn run_and_check_output<T: DeserializeOwned>(self) -> T {
    let stdout = self.stdout_regex(".*").run_and_extract_stdout();
    serde_json::from_str(&stdout)
      .unwrap_or_else(|err| panic!("Failed to deserialize JSON: {err}\n{stdout}"))
  }
}
