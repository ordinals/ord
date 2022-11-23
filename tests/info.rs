use super::*;

#[test]
fn basic() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("info")
    .rpc_server(&rpc_server)
    .stdout_regex(
      r#"\{"blocks_indexed":1,"branch_pages":\d+,"fragmented_bytes":\d+,"free_pages":\d+,"index_file_size":\d+,"leaf_pages":\d+,"metadata_bytes":\d+,"ordinal_ranges":1,"outputs_traversed":1,"page_size":\d+,"stored_bytes":\d+,"transactions":\[\{"starting_block_count":0,"starting_timestamp":\d+\}\],"tree_height":\d+,"utxos_indexed":1\}"#
    )
    .run();
}
