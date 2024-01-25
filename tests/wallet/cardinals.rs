use {
  super::*,
  ord::subcommand::wallet::{cardinals::CardinalUtxo, outputs::Output},
};

#[test]
fn cardinals() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let all_outputs = CommandBuilder::new("wallet outputs")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<Output>>();

  let cardinal_outputs = CommandBuilder::new("wallet cardinals")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<CardinalUtxo>>();

  assert_eq!(all_outputs.len() - cardinal_outputs.len(), 1);
}
