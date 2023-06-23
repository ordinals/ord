use {super::*, ord::subcommand::wallet::receive::Output};

#[test]
fn receive() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .run_and_check_output::<Output>();

  assert!(output.address.is_valid_for_network(Network::Bitcoin));
}
