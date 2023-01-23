use super::*;

#[test]
fn wallet_balance_only_counts_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  CommandBuilder::new("wallet balance")
    .rpc_server(&rpc_server)
    .expected_stdout("0\n")
    .run();

  inscribe(&rpc_server);

  CommandBuilder::new("wallet balance")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{}\n", 100 * COIN_VALUE - 10_000))
    .run();
}
