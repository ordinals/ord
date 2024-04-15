use {super::*, ord::subcommand::wallet::receive};

#[test]
fn receive() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn(&core);

  create_wallet(&core, &ord);

  let output = CommandBuilder::new("wallet receive")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<receive::Output>();

  assert!(output
    .addresses
    .first()
    .unwrap()
    .is_valid_for_network(Network::Bitcoin));
}
