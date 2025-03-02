use {super::*, ord::subcommand::wallet::receive};

#[test]
fn receive() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn(&core);

  let tempdir = create_wallet(&core, &ord);

  let output = CommandBuilder::new("wallet receive")
    .temp_dir(tempdir.clone())
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<receive::Output>();

  let first_address = output.addresses.first().unwrap();

  assert!(first_address.is_valid_for_network(Network::Bitcoin));

  let output = CommandBuilder::new("wallet receive")
    .temp_dir(tempdir)
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<receive::Output>();

  let second_address = output.addresses.first().unwrap();

  assert!(second_address != first_address);
}
