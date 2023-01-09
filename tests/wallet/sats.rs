use super::*;

#[test]
fn sats() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  let second_coinbase = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("--index-sats wallet sats")
    .rpc_server(&rpc_server)
    .expected_stdout(format!(
      "{}\t{}\t0\tuncommon\n",
      OutPoint::new(second_coinbase, 0),
      50 * COIN_VALUE,
    ))
    .run();
}

#[test]
fn sats_from_tsv_success() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  let second_coinbase = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("--index-sats wallet sats --tsv foo.tsv")
    .write("foo.tsv", "nvtcsezkbtg")
    .rpc_server(&rpc_server)
    .expected_stdout(format!(
      "{}\tnvtcsezkbtg\n",
      OutPoint::new(second_coinbase, 0),
    ))
    .run();
}

#[test]
fn sats_from_tsv_parse_error() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  CommandBuilder::new("wallet sats --tsv foo.tsv")
    .write("foo.tsv", "===")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr(
      "error: failed to parse sat from string \"===\" on line 1: invalid digit found in string\n",
    )
    .run();
}

#[test]
fn sats_from_tsv_file_not_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  CommandBuilder::new("wallet sats --tsv foo.tsv")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .stderr_regex("error: I/O error reading `.*`\nbecause: .*\n")
    .run();
}
