use {
  super::*,
  ord::subcommand::wallet::{cardinals::CardinalUtxo, outputs::Output},
};

#[test]
fn cardinals() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  inscribe(&rpc_server);

  let all_outputs = CommandBuilder::new("wallet outputs")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Vec<Output>>();

  let cardinal_outputs = CommandBuilder::new("wallet cardinals")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Vec<CardinalUtxo>>();

  assert_eq!(all_outputs.len() - cardinal_outputs.len(), 1);
}
