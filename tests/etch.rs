use {
  super::*,
  ord::{subcommand::wallet::etch::Output, Rune},
};

#[test]
fn flag_is_required() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  CommandBuilder::new(format!(
    "--regtest wallet etch --rune {} --divisibility 39 --fee-rate 1 --supply 1000 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(
    "error: `ord wallet etch` requires index created with `--index-runes-pre-alpha-i-agree-to-get-rekt` flag\n",
  )
  .run_and_extract_stdout();
}

#[test]
fn divisibility_over_max_is_an_error() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 39 --fee-rate 1 --supply 1000 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .expected_stderr("error: <DIVISBILITY> must be equal to or less than 38\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn supply_over_max_is_an_error() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 0 --fee-rate 1 --supply 340282366920938463463374607431768211456 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .stderr_regex(r"error: invalid value '\d+' for '--supply <SUPPLY>': number too large to fit in target type\n.*")
  .expected_exit_code(2)
  .run_and_extract_stdout();
}

#[test]
fn rune_below_minimum_is_an_error() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 0 --fee-rate 1 --supply 1000 --symbol ¢",
    Rune(RUNE - 1),
  ))
  .rpc_server(&rpc_server)
  .expected_stderr("error: rune is less than minimum for next block: ZZZZZZZZZZZZ < AAAAAAAAAAAAA\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn trying_to_etch_an_existing_rune_is_an_error() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  etch(&rpc_server, Rune(RUNE));

  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 0 --fee-rate 1 --supply 1000 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .expected_stderr("error: rune `AAAAAAAAAAAAA` has already been etched\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn runes_can_be_etched() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 1 --fee-rate 1 --supply 1000 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks(1);

  assert_eq!(
    runes(&rpc_server),
    vec![(
      Rune(RUNE),
      RuneInfo {
        burned: 0,
        divisibility: 1,
        end: None,
        etching: output.transaction,
        height: 2,
        id: RuneId {
          height: 2,
          index: 1
        },
        index: 1,
        limit: None,
        number: 0,
        rune: Rune(RUNE),
        supply: 1000,
        symbol: Some('¢'),
        timestamp: ord::timestamp(2),
      }
    )]
    .into_iter()
    .collect()
  );

  let output =
    CommandBuilder::new("--regtest --index-runes-pre-alpha-i-agree-to-get-rekt wallet balance")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(output.runes.unwrap()[&Rune(RUNE)], 1000);
}

#[test]
fn etch_sets_integer_fee_rate_correctly() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 1 --fee-rate 100 --supply 1000 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks(1);

  let tx = rpc_server.tx(2, 1);

  assert_eq!(tx.txid(), output.transaction);

  let output = tx.output.iter().map(|tx_out| tx_out.value).sum::<u64>();

  assert_eq!(output, 50 * COIN_VALUE - tx.vsize() as u64 * 100);
}

#[test]
fn etch_sets_decimal_fee_rate_correctly() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let output = CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 1 --fee-rate 100.5 --supply 1000 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks(1);

  let tx = rpc_server.tx(2, 1);

  assert_eq!(tx.txid(), output.transaction);

  let output = tx.output.iter().map(|tx_out| tx_out.value).sum::<u64>();

  assert_eq!(output, 50 * COIN_VALUE - (tx.vsize() as f64 * 100.5) as u64);
}

#[test]
fn etch_does_not_select_inscribed_utxos() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let output =
    CommandBuilder::new("--regtest --index-runes-pre-alpha-i-agree-to-get-rekt wallet balance")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(output.cardinal, 5000000000);

  CommandBuilder::new("--regtest wallet inscribe --fee-rate 0 --file foo.txt --postage 50btc")
    .write("foo.txt", "FOO")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks_with_subsidy(1, 0);

  let output =
    CommandBuilder::new("--regtest --index-runes-pre-alpha-i-agree-to-get-rekt wallet balance")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(output.cardinal, 0);

  CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 1 --fee-rate 1 --supply 1000 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .stderr_regex("error: JSON-RPC error: .*")
  .expected_exit_code(1)
  .run_and_extract_stdout();

  rpc_server.mine_blocks(1);
}

#[test]
fn inscribe_does_not_select_runic_utxos() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks_with_subsidy(1, 10000);

  CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 1 --fee-rate 0 --supply 1000 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks_with_subsidy(1, 0);

  let output =
    CommandBuilder::new("--regtest --index-runes-pre-alpha-i-agree-to-get-rekt wallet balance")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(output.cardinal, 0);
  assert_eq!(output.ordinal, 0);
  assert_eq!(output.runic, Some(10000));

  CommandBuilder::new("--regtest --index-runes-pre-alpha-i-agree-to-get-rekt wallet inscribe --fee-rate 0 --file foo.txt")
    .write("foo.txt", "FOO")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr("error: wallet contains no cardinal utxos\n")
    .run_and_extract_stdout();
}

#[test]
fn send_amount_does_not_select_runic_utxos() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks_with_subsidy(1, 10000);

  CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 1 --fee-rate 0 --supply 1000 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks_with_subsidy(1, 0);

  CommandBuilder::new("--regtest --index-runes-pre-alpha-i-agree-to-get-rekt wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 600sat")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .stderr_regex("error: JSON-RPC error: .*")
    .run_and_extract_stdout();
}

#[test]
fn send_satpoint_does_not_send_runic_utxos() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks_with_subsidy(1, 10000);

  let output = CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 1 --fee-rate 0 --supply 1000 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks_with_subsidy(1, 0);

  CommandBuilder::new(format!("--regtest --index-runes-pre-alpha-i-agree-to-get-rekt wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw {}:1:0", output.transaction))
    .rpc_server(&rpc_server)
    .expected_stderr("error: runic outpoints may not be sent by satpoint\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn send_inscription_does_not_select_runic_utxos() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);

  rpc_server.mine_blocks_with_subsidy(1, 10000);

  CommandBuilder::new(
    format!(
    "--index-runes-pre-alpha-i-agree-to-get-rekt --regtest wallet etch --rune {} --divisibility 1 --fee-rate 0 --supply 1000 --symbol ¢",
    Rune(RUNE),
  ))
  .rpc_server(&rpc_server)
  .run_and_deserialize_output::<Output>();

  rpc_server.mine_blocks_with_subsidy(1, 10000);

  let inscribe = CommandBuilder::new("--regtest --index-runes-pre-alpha-i-agree-to-get-rekt wallet inscribe --fee-rate 0 --file foo.txt")
    .write("foo.txt", "FOO")
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks_with_subsidy(1, 0);

  let output =
    CommandBuilder::new("--regtest --index-runes-pre-alpha-i-agree-to-get-rekt wallet balance")
      .rpc_server(&rpc_server)
      .run_and_deserialize_output::<ord::subcommand::wallet::balance::Output>();

  assert_eq!(output.cardinal, 0);
  assert_eq!(output.ordinal, 10000);
  assert_eq!(output.runic, Some(10000));

  CommandBuilder::new(format!("--regtest --index-runes-pre-alpha-i-agree-to-get-rekt wallet send --postage 10001sat --fee-rate 0 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw {}", inscribe.inscriptions[0].id))
    .rpc_server(&rpc_server)
    .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}
