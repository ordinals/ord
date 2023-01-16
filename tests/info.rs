use super::*;

#[test]
fn json_with_satoshi_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("--index-sats info")
    .rpc_server(&rpc_server)
    .stdout_regex(
      r#"\{
  "blocks_indexed": 1,
  "branch_pages": \d+,
  "fragmented_bytes": \d+,
  "index_file_size": \d+,
  "index_path": ".*\.redb",
  "leaf_pages": \d+,
  "metadata_bytes": \d+,
  "outputs_traversed": 1,
  "page_size": \d+,
  "sat_ranges": 1,
  "stored_bytes": \d+,
  "transactions": \[
    \{
      "starting_block_count": 0,
      "starting_timestamp": \d+
    \}
  \],
  "tree_height": \d+,
  "utxos_indexed": 2
\}
"#,
    )
    .run();
}

#[test]
fn json_without_satoshi_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("info")
    .rpc_server(&rpc_server)
    .stdout_regex(
      r#"\{
  "blocks_indexed": 1,
  "branch_pages": \d+,
  "fragmented_bytes": \d+,
  "index_file_size": \d+,
  "index_path": ".*\.redb",
  "leaf_pages": \d+,
  "metadata_bytes": \d+,
  "outputs_traversed": 0,
  "page_size": \d+,
  "sat_ranges": 0,
  "stored_bytes": \d+,
  "transactions": \[
    \{
      "starting_block_count": 0,
      "starting_timestamp": \d+
    \}
  \],
  "tree_height": \d+,
  "utxos_indexed": 0
\}
"#,
    )
    .run();
}

#[test]
fn transactions() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let tempdir = TempDir::new().unwrap();

  let index_path = tempdir.path().join("index.redb");

  CommandBuilder::new(format!(
    "--index {} info --transactions",
    index_path.display()
  ))
  .rpc_server(&rpc_server)
  .expected_stdout("start\tend\tcount\telapsed\n")
  .run();

  rpc_server.mine_blocks(10);

  CommandBuilder::new(format!(
    "--index {} info --transactions",
    index_path.display()
  ))
  .rpc_server(&rpc_server)
  .stdout_regex("start\tend\tcount\telapsed\n0\t1\t1\t\\d+\\.\\d+\n")
  .run();

  rpc_server.mine_blocks(10);

  CommandBuilder::new(format!(
    "--index {} info --transactions",
    index_path.display()
  ))
  .rpc_server(&rpc_server)
  .expected_stdout("start\tend\tcount\telapsed\n")
  .stdout_regex("start\tend\tcount\telapsed\n0\t1\t1\t\\d+\\.\\d+\n")
  .stdout_regex("start\tend\tcount\telapsed\n0\t1\t1\t\\d+\\.\\d+\n1\t11\t10\t\\d+\\.\\d+\n")
  .run();
}
