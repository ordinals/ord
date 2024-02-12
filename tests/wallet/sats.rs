use {
  super::*,
  ord::subcommand::wallet::sats::{OutputRare, OutputTsv},
};

#[test]
fn requires_sat_index() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new("wallet sats")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: sats requires index created with `--index-sats` flag\n")
    .run_and_extract_stdout();
}

#[test]
fn sats() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let second_coinbase = bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let output = CommandBuilder::new("--index-sats wallet sats")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<OutputRare>>();

  assert_eq!(output[0].sat, 50 * COIN_VALUE);
  assert_eq!(output[0].output.to_string(), format!("{second_coinbase}:0"));
}

#[test]
fn sats_from_tsv_success() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let second_coinbase = bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let output = CommandBuilder::new("--index-sats wallet sats --tsv foo.tsv")
    .write("foo.tsv", "nvtcsezkbtg")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .run_and_deserialize_output::<Vec<OutputTsv>>();

  assert_eq!(output[0].sat, "nvtcsezkbtg");
  assert_eq!(output[0].output.to_string(), format!("{second_coinbase}:0"));
}

#[test]
fn sats_from_tsv_parse_error() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new("--index-sats wallet sats --tsv foo.tsv")
    .write("foo.tsv", "===")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .expected_stderr(
      "error: failed to parse sat from string \"===\" on line 1: failed to parse sat `===`: invalid integer: invalid digit found in string\n",
    )
    .run_and_extract_stdout();
}

#[test]
fn sats_from_tsv_file_not_found() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  CommandBuilder::new("--index-sats wallet sats --tsv foo.tsv")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .ord_rpc_server(&ord_rpc_server)
    .expected_exit_code(1)
    .stderr_regex("error: I/O error reading `.*`\nbecause: .*\n")
    .run_and_extract_stdout();
}
