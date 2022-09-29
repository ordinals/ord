use super::*;

#[test]
fn output_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("list 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0")
    .rpc_server(&rpc_server)
    .expected_stdout("[0,5000000000)\n")
    .run();
}

#[test]
fn output_not_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("list 150ba822b458a19615e70a604d8dd9d3482fc165fa4e9cc150d74e11916ce8ae:0")
    .rpc_server(&rpc_server)
    .expected_status(1)
    .run();
}
