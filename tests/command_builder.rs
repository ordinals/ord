use super::*;

enum ExpectedExitStatus {
  Code(i32),
  Signal(Signal),
}

pub(crate) struct CommandBuilder {
  args: &'static str,
  expected_exit_status: ExpectedExitStatus,
  expected_stderr: Expected,
  expected_stdout: Expected,
  rpc_server_url: Option<String>,
  tempdir: TempDir,
}

impl CommandBuilder {
  pub(crate) fn new(args: &'static str) -> Self {
    Self {
      args,
      expected_exit_status: ExpectedExitStatus::Code(0),
      expected_stderr: Expected::String(String::new()),
      expected_stdout: Expected::String(String::new()),
      rpc_server_url: None,
      tempdir: TempDir::new().unwrap(),
    }
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

  pub(crate) fn expected_exit_code(self, expected_status: i32) -> Self {
    Self {
      expected_exit_status: ExpectedExitStatus::Code(expected_status),
      ..self
    }
  }

  pub(crate) fn expected_exit_signal(self, expected_signal: Signal) -> Self {
    Self {
      expected_exit_status: ExpectedExitStatus::Signal(expected_signal),
      ..self
    }
  }

  pub(crate) fn command(&self) -> Command {
    let mut command = Command::new(executable_path("ord"));

    if let Some(rpc_server_url) = &self.rpc_server_url {
      let cookiefile = self.tempdir.path().join("cookie");
      fs::write(&cookiefile, "username:password").unwrap();
      command.args(&[
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
      .args(self.args.split_whitespace());

    command
  }

  pub(crate) fn check(self, output: Output) -> TempDir {
    let stdout = str::from_utf8(&output.stdout).unwrap();
    let stderr = str::from_utf8(&output.stderr).unwrap();

    if match self.expected_exit_status {
      ExpectedExitStatus::Code(code) => output.status.code() != Some(code),
      ExpectedExitStatus::Signal(signal) => output.status.signal() != Some(signal as i32),
    } {
      panic!(
        "Test failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status, stdout, stderr
      );
    }

    self.expected_stderr.assert_match(stderr);
    self.expected_stdout.assert_match(stdout);

    self.tempdir
  }

  pub(crate) fn run(self) -> TempDir {
    let output = self.command().output().unwrap();
    self.check(output)
  }
}
