use {super::*, std::str::FromStr};

#[test]
fn satoshis() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  let second_coinbase = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("--index-satoshis wallet satoshis")
    .rpc_server(&rpc_server)
    .expected_stdout(format!(
      "{}\t{}\t0\tuncommon\n",
      OutPoint::new(second_coinbase, 0),
      50 * COIN_VALUE,
    ))
    .run();
}

#[test]
fn satoshis_from_tsv_success() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  let second_coinbase = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("--index-satoshis wallet satoshis --tsv foo.tsv")
    .write("foo.tsv", "nvtcsezkbtg")
    .rpc_server(&rpc_server)
    .expected_stdout(format!(
      "{}\tnvtcsezkbtg\n",
      OutPoint::new(second_coinbase, 0),
    ))
    .run();
}

#[test]
fn satoshis_from_tsv_parse_error() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("wallet satoshis --tsv foo.tsv")
    .write("foo.tsv", "===")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .expected_stderr(
      "error: failed to parse sat from string \"===\" on line 1: invalid digit found in string\n",
    )
    .run();
}

#[test]
fn satoshis_from_tsv_file_not_found() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  CommandBuilder::new("wallet satoshis --tsv foo.tsv")
    .rpc_server(&rpc_server)
    .expected_exit_code(1)
    .stderr_regex("error: I/O error reading `.*`\nbecause: .*\n")
    .run();
}

#[test]
fn send_works_on_signet() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Signet, "ord");

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain signet --index-satoshis wallet inscribe --satpoint {txid}:0:0 --file degenerate.png"
  ))
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new(format!(
    "--chain signet wallet send {reveal_txid} tb1qx4gf3ya0cxfcwydpq8vr2lhrysneuj5d7lqatw"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{}\n", txid), stdout);

  rpc_server.mine_blocks(1);

  let send_txid = stdout.trim();

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    &format!("/inscription/{reveal_txid}"),
    &format!(
      ".*<h1>Inscription {reveal_txid}</h1>
<dl>
  <dt>location</dt>
  <dd>{send_txid}:0:0</dd>
</dl>
.*",
    ),
  );
}

#[test]
fn send_unknown_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Signet, "ord");

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain signet wallet send {txid} tb1qx4gf3ya0cxfcwydpq8vr2lhrysneuj5d7lqatw"
  ))
  .rpc_server(&rpc_server)
  .expected_stderr(format!("error: No inscription found for {txid}\n"))
  .expected_exit_code(1)
  .run();
}

#[test]
fn send_inscribed_sat() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Signet, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain signet --index-satoshis wallet inscribe --satpoint {txid}:0:0 --file degenerate.png"
  ))
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  rpc_server.mine_blocks(1);

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  let stdout = CommandBuilder::new(format!(
    "--chain signet wallet send {reveal_txid} tb1qx4gf3ya0cxfcwydpq8vr2lhrysneuj5d7lqatw"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex("[[:xdigit:]]{64}\n")
  .run();

  rpc_server.mine_blocks(1);

  let send_txid = stdout.trim();

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    &format!("/inscription/{reveal_txid}"),
    &format!(
      ".*<h1>Inscription {reveal_txid}</h1>
<dl>
  <dt>location</dt>
  <dd>{send_txid}:0:0</dd>
</dl>
.*",
    ),
  );
}

#[test]
fn send_on_mainnnet_refuses_to_work_with_wallet_name_foo() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Bitcoin, "foo");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(
    format!("wallet send {txid}:0:0 bc1qzjeg3h996kw24zrg69nge97fw8jc4v7v7yznftzk06j3429t52vse9tkp9"),
  )
  .rpc_server(&rpc_server)
  .expected_stderr("error: `ord wallet send` may only be used on mainnet with a wallet named `ord` or whose name starts with `ord-`\n")
  .expected_exit_code(1)
  .run();
}

#[test]
fn send_addresses_must_be_valid_for_network() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Bitcoin, "ord");
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "wallet send {txid}:0:0 tb1qx4gf3ya0cxfcwydpq8vr2lhrysneuj5d7lqatw"
  ))
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
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "wallet send {txid}:0:0 bc1qzjeg3h996kw24zrg69nge97fw8jc4v7v7yznftzk06j3429t52vse9tkp9"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{}\n", txid), stdout);
}

#[test]
fn send_on_mainnnet_works_with_wallet_whose_name_starts_with_ord() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Bitcoin, "ord-foo");
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "wallet send {txid}:0:0 bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{}\n", txid), stdout);
}

#[test]
fn send_on_mainnnet_refuses_to_work_with_wallet_with_high_balance() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Bitcoin, "ord");
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000_001)[0].txdata[0].txid();

  CommandBuilder::new(format!("wallet send {txid}:0:0 bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"))
    .rpc_server(&rpc_server)
    .expected_stderr(
      "error: `ord wallet send` may not be used on mainnet with wallets containing more than 1,000,000 sats\n",
    )
    .expected_exit_code(1)
    .run();
}

#[test]
fn inscribe() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  rpc_server.mine_blocks(1);

  TestServer::spawn_with_args(&rpc_server, &["--index-satoshis"]).assert_response_regex(
    "/sat/5000000000",
    ".*<dt>inscription</dt>\n  <dd>HELLOWORLD</dd>.*",
  );

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    &format!("/inscription/{}", reveal_txid_from_inscribe_stdout(&stdout)),
    ".*HELLOWORLD.*",
  );
}

#[test]
fn inscribe_forbidden_on_mainnet() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Bitcoin, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {txid}:0:0 --file hello.txt"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: `ord wallet inscribe` is unstable and not yet supported on mainnet.\n")
  .run();
}

#[test]
fn inscribe_unknown_file_extension() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file pepe.jpg"
  ))
  .write("pepe.jpg", [1; 520])
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: unrecognized file extension `.jpg`, only .txt and .png accepted\n")
  .run();
}

#[test]
fn inscribe_png() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest --index-satoshis wallet inscribe --satpoint {txid}:0:0 --file degenerate.png"
  ))
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &["--index-satoshis"]);

  ord_server.assert_response_regex(
    "/sat/5000000000",
    ".*<dt>inscription</dt>\n  <dd><img src=.*",
  )
}

#[test]
fn inscribe_exceeds_push_byte_limit() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Signet, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain signet wallet inscribe --satpoint {txid}:0:0 --file degenerate.png"
  ))
  .write("degenerate.png", [1; 1025])
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(
    "error: content size of 1025 bytes exceeds 1024 byte limit for signet inscriptions\n",
  )
  .run();
}

#[test]
fn regtest_has_no_content_size_limit() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file degenerate.png"
  ))
  .write("degenerate.png", [1; 1025])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();
}

#[test]
fn inscribe_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  let txid = rpc_server.mine_blocks_with_subsidy(1, 800)[0].txdata[0].txid();
  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file degenerate.png"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let txid = rpc_server.mine_blocks_with_subsidy(1, 100)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file degenerate.png"
  ))
  .rpc_server(&rpc_server)
  .write("degenerate.png", [1; 100])
  .expected_exit_code(1)
  .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
  .run();
}

#[test]
fn send_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  let txid = rpc_server.mine_blocks_with_subsidy(1, 800)[0].txdata[0].txid();
  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file degenerate.png"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let txid = rpc_server.mine_blocks_with_subsidy(1, 100)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest wallet send {txid}:0:0 bcrt1q6rhpng9evdsfnn833a4f4vej0asu6dk5srld6x"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
  .run();
}

#[test]
fn refuse_to_reinscribe_sats() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");

  let txid = rpc_server.mine_blocks_with_subsidy(1, 800)[0].txdata[0].txid();
  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file degenerate.png"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let first_inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks_with_subsidy(1, 100)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {first_inscription_id}:0:0 --file hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: sat at {first_inscription_id}:0:0 already inscribed\n"
  ))
  .run();
}

#[test]
fn do_not_accidentally_send_an_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();
  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file degenerate.png"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  let inscription_utxo = OutPoint {
    txid: reveal_txid_from_inscribe_stdout(&stdout),
    vout: 0,
  };

  CommandBuilder::new(format!(
    "--chain regtest wallet send {inscription_utxo}:55 bcrt1q6rhpng9evdsfnn833a4f4vej0asu6dk5srld6x"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: cannot send {inscription_utxo}:55 without also sending inscription {inscription_id} at {inscription_utxo}:0\n"
  ))
  .run();
}

#[test]
fn refuse_to_inscribe_already_inscribed_utxo() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();
  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file degenerate.png"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  rpc_server.mine_blocks(1);

  let inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

  let inscription_utxo = OutPoint {
    txid: reveal_txid_from_inscribe_stdout(&stdout),
    vout: 0,
  };

  CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {inscription_utxo}:55555 --file hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: utxo {inscription_utxo} already inscribed with inscription {inscription_id} on sat {inscription_utxo}:0\n",
  ))
  .run();
}

#[test]
fn inscriptions_cannot_be_sent_by_satpoint() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain regtest wallet inscribe --satpoint {txid}:0:0 --file hello.txt"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(format!(
    "--chain regtest wallet send {reveal_txid}:0:0 bcrt1q6rhpng9evdsfnn833a4f4vej0asu6dk5srld6x"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_stderr("error: inscriptions must be sent by inscription ID\n")
  .expected_exit_code(1)
  .run();
}

#[test]
fn receive() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let stdout = CommandBuilder::new("wallet receive")
    .rpc_server(&rpc_server)
    .stdout_regex(".*")
    .run();

  assert!(Address::from_str(stdout.trim()).is_ok());
}

#[test]
fn utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let coinbase_tx = &rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0];
  let outpoint = OutPoint::new(coinbase_tx.txid(), 0);
  let amount = coinbase_tx.output[0].value;

  CommandBuilder::new("wallet utxos")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{outpoint}\t{amount}\n"))
    .run();
}

#[test]
fn inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Signet, "ord-wallet");
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let inscription_id = reveal_txid_from_inscribe_stdout(
    &CommandBuilder::new(format!(
      "--chain signet wallet inscribe --satpoint {txid}:0:0 --file hello.txt"
    ))
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run(),
  );

  rpc_server.mine_blocks(1);

  CommandBuilder::new("--chain signet wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription_id}\t{inscription_id}:0:0\n"))
    .run();

  let stdout = CommandBuilder::new("--chain signet wallet receive")
    .rpc_server(&rpc_server)
    .expected_exit_code(0)
    .stdout_regex(".*")
    .run();

  let address = stdout.trim();

  let stdout = CommandBuilder::new(format!(
    "--chain signet wallet send {inscription_id} {address}"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(0)
  .stdout_regex(".*")
  .run();

  rpc_server.mine_blocks(1);

  let txid = Txid::from_str(stdout.trim()).unwrap();

  let outpoint = OutPoint::new(txid, 0);

  CommandBuilder::new("--chain signet wallet inscriptions")
    .rpc_server(&rpc_server)
    .expected_stdout(format!("{inscription_id}\t{outpoint}:0\n"))
    .run();
}

#[test]
fn inscribe_with_optional_satpoint_arg() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");
  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new("--chain regtest wallet inscribe --file hello.txt")
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  rpc_server.mine_blocks(1);

  TestServer::spawn_with_args(&rpc_server, &["--index-satoshis"]).assert_response_regex(
    "/sat/5000000000",
    ".*<dt>inscription</dt>\n  <dd>HELLOWORLD</dd>.*",
  );

  TestServer::spawn_with_args(&rpc_server, &[]).assert_response_regex(
    &format!("/inscription/{}", reveal_txid_from_inscribe_stdout(&stdout)),
    ".*HELLOWORLD.*",
  );
}

#[test]
fn create() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");

  assert!(!rpc_server.wallets().contains("ord"));

  CommandBuilder::new("--chain regtest wallet create")
    .rpc_server(&rpc_server)
    .run();

  assert!(rpc_server.wallets().contains("ord"));
}

#[test]
fn transactions() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Signet, "ord");
  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new("--chain signet wallet inscribe --file degenerate.png")
    .write("degenerate.png", [1; 520])
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  CommandBuilder::new("--chain signet wallet transactions")
    .rpc_server(&rpc_server)
    .stdout_regex(format!(".*{}\t0.*", reveal_txid))
    .run();

  rpc_server.mine_blocks(1);

  CommandBuilder::new("--chain signet wallet transactions")
    .rpc_server(&rpc_server)
    .stdout_regex(format!(".*{}\t1\n.*", reveal_txid))
    .run();

  let txid = CommandBuilder::new(format!(
    "--chain signet wallet send {reveal_txid} tb1qx4gf3ya0cxfcwydpq8vr2lhrysneuj5d7lqatw"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run();

  CommandBuilder::new("--chain signet wallet transactions")
    .rpc_server(&rpc_server)
    .stdout_regex(format!(".*{}\t0\n.*", txid.trim()))
    .run();
}
