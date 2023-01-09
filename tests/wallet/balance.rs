use super::*;

#[test]
fn wallet_balance() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();
  create_wallet(&rpc_server);

  CommandBuilder::new("--regtest wallet balance")
    .rpc_server(&rpc_server)
    .expected_stdout("0\n")
    .run();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("--regtest wallet balance")
    .rpc_server(&rpc_server)
    .expected_stdout("5000000000\n")
    .run();
}
