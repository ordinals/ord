use super::*;

pub(crate) struct Output {
  pub(crate) tempdir: TempDir,
  pub(crate) stdout: String,
}

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
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .env("HOME", self.tempdir.path())
      .current_dir(&self.tempdir)
      .args(&self.args);

    command
  }

  pub(crate) fn check(self, output: process::Output) -> TempDir {
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

    self.tempdir
  }

  pub(crate) fn run(self) -> Output {
    let output = self.command().output().unwrap();
    let stdout = String::from(str::from_utf8(&output.stdout).unwrap());

    Output {
      tempdir: self.check(output),
      stdout,
    }
  }
}
