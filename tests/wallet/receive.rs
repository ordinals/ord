use {super::*, ord::subcommand::wallet::receive};

#[test]
fn receive() {
  let bitcoin_rpc_server = mockcore::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let output = CommandBuilder::new("wallet receive")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<receive::Output>();

  assert!(output
    .addresses
    .first()
    .unwrap()
    .is_valid_for_network(Network::Bitcoin));
}
