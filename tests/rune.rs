use {
  super::*,
  std::{net::TcpListener, process::Child, thread, time::Duration},
};

struct KillOnDrop(Child);

impl Drop for KillOnDrop {
  fn drop(&mut self) {
    self.0.kill().unwrap()
  }
}

#[test]
fn publish() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let port = TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();

  let tempdir = TempDir::new().unwrap();

  fs::create_dir(tempdir.path().join("regtest")).unwrap();
  fs::write(tempdir.path().join("regtest/.cookie"), "foo:bar").unwrap();

  let _ord_server = KillOnDrop(CommandBuilder::new(format!(
    "--chain regtest --rpc-url {} --bitcoin-data-dir {} --data-dir {} server --http-port {port} --address 127.0.0.1",
    rpc_server.url(),
    tempdir.path().display(),
    tempdir.path().display()
  ))
  .command()
  .spawn()
  .unwrap());

  thread::sleep(Duration::from_secs(1));

  CommandBuilder::new(format!(
    "--chain regtest rune publish --name foo --ordinal 0 --url http://127.0.0.1:{port}"
  ))
  .rpc_server(&rpc_server)
  .run();
}

#[test]
fn publish_mainnet_forbidden() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new("rune publish --name foo --ordinal 0")
    .rpc_server(&rpc_server)
    .expected_stderr("error: `ord rune publish` is unstable and not yet supported on mainnet.\n")
    .expected_exit_code(1)
    .run();
}
