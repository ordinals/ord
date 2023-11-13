use {
  super::*,
  ord::subcommand::wallet::{inscriptions, receive, send},
};

#[test]
fn inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let (inscription, reveal) = inscribe(&rpc_server);

  let output = CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription);
  assert_eq!(output[0].location, format!("{reveal}:0:0").parse().unwrap());
  assert_eq!(
    output[0].explorer,
    format!("https://ordinals.com/inscription/{inscription}")
  );

  let address = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<receive::Output>()
    .address;

  let txid = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 {} {inscription}",
    address.assume_checked()
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(0)
  .stdout_regex(".*")
  .run_and_deserialize_output::<send::Output>()
  .transaction;

  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription);
  assert_eq!(output[0].location, format!("{txid}:0:0").parse().unwrap());
}

#[test]
fn inscriptions_includes_locked_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let (inscription, reveal) = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  rpc_server.lock(OutPoint {
    txid: reveal,
    vout: 0,
  });

  let output = CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription);
  assert_eq!(output[0].location, format!("{reveal}:0:0").parse().unwrap());
}

#[test]
fn inscriptions_with_postage() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&rpc_server);

  let output = CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output[0].postage, 10000);

  let address = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<receive::Output>()
    .address;

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 {} {inscription}",
    address.assume_checked()
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(0)
  .stdout_regex(".*")
  .run_and_extract_stdout();

  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output[0].postage, 9889);
}
