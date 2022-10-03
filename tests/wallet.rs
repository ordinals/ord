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
