use {super::*, ord::subcommand::wallet::dump};

#[test]
fn dumped_descriptors_match_wallet() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let output = CommandBuilder::new("wallet dump")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .stderr_regex(".*")
    .run_and_deserialize_output::<dump::Output>();

  assert!(bitcoin_rpc_server
    .descriptors()
    .iter()
    .zip(output.descriptors.iter())
    .all(|(wallet_descriptor, output_descriptor)| *wallet_descriptor == output_descriptor.desc));
}
