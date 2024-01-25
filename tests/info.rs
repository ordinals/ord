use {super::*, ord::subcommand::index::info::TransactionsOutput};

#[test]
fn json_with_satoshi_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("--index-sats index info")
    .bitcoin_rpc_server(&rpc_server)
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
  "tables": .*,
  "total_bytes": \d+,
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
    .run_and_extract_stdout();
}

#[test]
fn json_without_satoshi_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("index info")
    .bitcoin_rpc_server(&rpc_server)
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
  "tables": .*,
  "total_bytes": \d+,
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
    .run_and_extract_stdout();
}

#[test]
fn transactions() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let tempdir = TempDir::new().unwrap();

  let index_path = tempdir.path().join("index.redb");

  assert!(CommandBuilder::new(format!(
    "--index {} index info --transactions",
    index_path.display()
  ))
  .bitcoin_rpc_server(&rpc_server)
  .run_and_deserialize_output::<Vec<TransactionsOutput>>()
  .is_empty());

  rpc_server.mine_blocks(10);

  let output = CommandBuilder::new(format!(
    "--index {} index info --transactions",
    index_path.display()
  ))
  .bitcoin_rpc_server(&rpc_server)
  .run_and_deserialize_output::<Vec<TransactionsOutput>>();

  assert_eq!(output[0].start, 0);
  assert_eq!(output[0].end, 1);
  assert_eq!(output[0].count, 1);

  rpc_server.mine_blocks(10);

  let output = CommandBuilder::new(format!(
    "--index {} index info --transactions",
    index_path.display()
  ))
  .bitcoin_rpc_server(&rpc_server)
  .run_and_deserialize_output::<Vec<TransactionsOutput>>();

  assert_eq!(output[1].start, 1);
  assert_eq!(output[1].end, 11);
  assert_eq!(output[1].count, 10);
}
