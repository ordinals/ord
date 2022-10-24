use super::*;

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
fn send_works_on_signet() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Signet, "ord");

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
fn send_on_mainnnet_refuses_to_work_with_wallet_name_foo() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Bitcoin, "foo");
  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "wallet send 5000000000 bc1qzjeg3h996kw24zrg69nge97fw8jc4v7v7yznftzk06j3429t52vse9tkp9",
  )
  .rpc_server(&rpc_server)
  .expected_stderr("error: `ord wallet send` may only be used on mainnet with a wallet named `ord` or whose name starts with `ord-`\n")
  .expected_exit_code(1)
  .run();
}

#[test]
fn send_addresses_must_be_valid_for_network() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Bitcoin, "ord");
  rpc_server.mine_blocks_with_subsidy(1, 1_000_000);

  CommandBuilder::new("wallet send 5000000000 tb1qx4gf3ya0cxfcwydpq8vr2lhrysneuj5d7lqatw")
    .rpc_server(&rpc_server)
    .expected_stderr(
      "error: Address `tb1qx4gf3ya0cxfcwydpq8vr2lhrysneuj5d7lqatw` is not valid for mainnet\n",
    )
    .expected_exit_code(1)
    .run();
}

#[test]
fn send_on_mainnnet_works_with_wallet_named_ord() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Bitcoin, "ord");
  rpc_server.mine_blocks_with_subsidy(1, 1_000_000);

  let output =
    CommandBuilder::new("wallet send 5000000000 bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")
      .rpc_server(&rpc_server)
      .stdout_regex(r".*")
      .run();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{}\n", txid), output.stdout)
}

#[test]
fn send_on_mainnnet_works_with_wallet_whose_name_starts_with_ord() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Bitcoin, "ord-foo");
  rpc_server.mine_blocks_with_subsidy(1, 1_000_000);

  let output =
    CommandBuilder::new("wallet send 5000000000 bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")
      .rpc_server(&rpc_server)
      .stdout_regex(r".*")
      .run();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{}\n", txid), output.stdout)
}

#[test]
fn send_on_mainnnet_refuses_to_work_with_wallet_with_high_balance() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Bitcoin, "ord");
  rpc_server.mine_blocks_with_subsidy(1, 1_000_001);

  CommandBuilder::new("wallet send 5000000000 bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh")
    .rpc_server(&rpc_server)
    .expected_stderr(
      "error: `ord wallet send` may not be used on mainnet with wallets containing more than 1,000,000 sats\n",
    )
    .expected_exit_code(1)
    .run();
}
