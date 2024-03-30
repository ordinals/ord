use {
  super::*,
  ord::subcommand::wallet::{cardinals::CardinalUtxo, outputs::Output},
};

#[test]
fn cardinals() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  inscribe(&core, &ord);

  let all_outputs = CommandBuilder::new("wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<Output>>();

  let cardinal_outputs = CommandBuilder::new("wallet cardinals")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<CardinalUtxo>>();

  assert_eq!(all_outputs.len() - cardinal_outputs.len(), 1);
}
