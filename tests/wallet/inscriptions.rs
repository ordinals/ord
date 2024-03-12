use {
  super::*,
  ord::subcommand::wallet::{inscriptions, receive, send},
};

#[test]
fn inscriptions() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (inscription, reveal) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let output = CommandBuilder::new("wallet inscriptions")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription);
  assert_eq!(output[0].location, format!("{reveal}:0:0").parse().unwrap());
  assert_eq!(
    output[0].explorer,
    format!("https://ordinals.com/inscription/{inscription}")
  );

  let address = CommandBuilder::new("wallet receive")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<receive::Output>()
    .address;

  let txid = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 {} {inscription}",
    address.assume_checked()
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(0)
  .stdout_regex(".*")
  .run_and_deserialize_output::<send::Output>()
  .txid;

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription);
  assert_eq!(output[0].location, format!("{txid}:0:0").parse().unwrap());
}

#[test]
fn inscriptions_includes_locked_utxos() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (inscription, reveal) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  bitcoin_rpc_server.lock(OutPoint {
    txid: reveal,
    vout: 0,
  });

  let output = CommandBuilder::new("wallet inscriptions")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output.len(), 1);
  assert_eq!(output[0].inscription, inscription);
  assert_eq!(output[0].location, format!("{reveal}:0:0").parse().unwrap());
}

#[test]
fn inscriptions_with_postage() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (inscription, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let output = CommandBuilder::new("wallet inscriptions")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output[0].postage, 10000);

  let address = CommandBuilder::new("wallet receive")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<receive::Output>()
    .address;

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 {} {inscription}",
    address.assume_checked()
  ))
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .expected_exit_code(0)
  .stdout_regex(".*")
  .run_and_extract_stdout();

  bitcoin_rpc_server.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<inscriptions::Output>>();

  assert_eq!(output[0].postage, 9889);
}
