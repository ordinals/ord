use super::*;

#[test]
fn custom_index_size() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("--max-index-size 1mib find 0")
    .rpc_server(&rpc_server)
    .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0\n")
    .run();

  assert_eq!(
    output
      .tempdir
      .path()
      .join(if cfg!(target_os = "macos") {
        "Library/Application Support/"
      } else {
        ".local/share"
      })
      .join("ord")
      .join("index.redb")
      .metadata()
      .unwrap()
      .len(),
    1 << 20
  );
}
