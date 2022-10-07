use {
  super::*,
  bitcoin::{blockdata::constants::COIN_VALUE, OutPoint},
};

#[test]
fn identify() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  let second_coinbase = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("wallet identify")
    .rpc_server(&rpc_server)
    .expected_stdout(format!(
      "{}\t{}\t0\tuncommon\n",
      OutPoint::new(second_coinbase, 0),
      50 * COIN_VALUE,
    ))
    .run();
}

#[test]
fn list() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  let second_coinbase = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("wallet list")
    .rpc_server(&rpc_server)
    .expected_stdout(format!(
      "{}\t{}\t{}\tuncommon\tnvtcsezkbth\n",
      OutPoint::new(second_coinbase, 0),
      50 * COIN_VALUE,
      50 * COIN_VALUE,
    ))
    .run();
}

#[test]
fn send() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let output = CommandBuilder::new(
    "--chain signet wallet send 5000000000 tb1qx4gf3ya0cxfcwydpq8vr2lhrysneuj5d7lqatw",
  )
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{}\n", txid), output.stdout)
}

#[test]
fn send_not_allowed_on_mainnet() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("wallet send 5000000000 tb1qx4gf3ya0cxfcwydpq8vr2lhrysneuj5d7lqatw")
    .rpc_server(&rpc_server)
    .expected_stderr(
      "error: Send command is not allowed on mainnet yet. Try on regtest/signet/testnet.\n",
    )
    .expected_exit_code(1)
    .run();
}
