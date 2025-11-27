use {super::*, ord::subcommand::wallet::addresses::Output};

#[test]
fn addresses() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest"], &[]);

  create_wallet(&core, &ord);

  let output = CommandBuilder::new("--regtest wallet addresses")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<BTreeMap<Address<NetworkUnchecked>, Vec<Output>>>();

  pretty_assert_eq!(
    output
      .get(&etched.output.inscriptions[0].destination)
      .unwrap(),
    &vec![Output {
      output: etched.output.inscriptions[0].location.outpoint,
      amount: 10000,
      inscriptions: Some(vec![etched.output.inscriptions[0].id]),
    }]
  );
}
