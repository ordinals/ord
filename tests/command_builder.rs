use super::*;

pub(crate) struct CommandBuilder {
  args: &'static str,
  expected_status: i32,
  expected_stderr: Expected,
  expected_stdout: Expected,
  rpc_server_url: Option<String>,
  tempdir: TempDir,
}

impl CommandBuilder {
  pub(crate) fn new(args: &'static str) -> Self {
    Self {
      args,
      expected_status: 0,
      expected_stderr: Expected::Ignore,
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

  pub(crate) fn run(self) {
    let mut command = Command::new(executable_path("ord"));

    if let Some(rpc_server_url) = self.rpc_server_url {
      let cookiefile = self.tempdir.path().join("cookie");
      fs::write(&cookiefile, "username:password").unwrap();
      command.args(&[
        "--rpc-url",
        &rpc_server_url,
        "--cookie-file",
        cookiefile.to_str().unwrap(),
      ]);
    }

    let output = command
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(if !matches!(self.expected_stderr, Expected::Ignore) {
        Stdio::piped()
      } else {
        Stdio::inherit()
      })
      .env("HOME", self.tempdir.path())
      .current_dir(&self.tempdir)
      .args(self.args.split_whitespace())
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

    self.expected_stderr.assert_match(stderr);
    self.expected_stdout.assert_match(stdout);
  }
}
