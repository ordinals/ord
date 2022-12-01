use super::*;

#[test]
fn output_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new(
    "--index-ordinals list 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0",
  )
  .rpc_server(&rpc_server)
  .expected_stdout("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0\t0\t5000000000\tmythic\tnvtdijuwxlp\n")
  .run();
}

#[test]
fn output_not_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new(
    "--index-ordinals list 0000000000000000000000000000000000000000000000000000000000000000:0",
  )
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: output not found\n")
  .run();
}

#[test]
fn no_ordinal_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("list 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0")
    .rpc_server(&rpc_server)
    .expected_stderr("error: list requires index created with `--index-ordinals` flag\n")
    .expected_exit_code(1)
    .run();
}
