use super::*;

#[test]
fn send_works() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--index-sats wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new(format!(
    "wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw {reveal_txid}"
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
    format!("/inscription/{reveal_txid}"),
    format!(
      ".*<h1>Inscription {reveal_txid}</h1>.*<dl>.*
  <dt>content size</dt>
  <dd>520 bytes</dd>
  <dt>content type</dt>
  <dd>image/png</dd>
  .*
  <dt>location</dt>
  <dd class=monospace>{send_txid}:0:0</dd>
  .*
</dl>
.*",
    ),
  );
}

#[test]
fn send_unknown_inscription() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "--chain signet wallet send tord1q497kurvh0fgtedca5angel7j4rdwe0q8h925u0 {txid}"
  ))
  .rpc_server(&rpc_server)
  .expected_stderr(format!("error: No inscription found for {txid}\n"))
  .expected_exit_code(1)
  .run();
}

#[test]
fn send_inscribed_sat() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Signet)
    .build();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "--chain signet --index-sats wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .write("degenerate.png", [1; 520])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  rpc_server.mine_blocks(1);

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  let stdout = CommandBuilder::new(format!(
    "--chain signet wallet send tord1q497kurvh0fgtedca5angel7j4rdwe0q8h925u0 {reveal_txid}"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex("[[:xdigit:]]{64}\n")
  .run();

  rpc_server.mine_blocks(1);

  let send_txid = stdout.trim();

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    format!("/inscription/{reveal_txid}"),
    format!(
      ".*<h1>Inscription {reveal_txid}</h1>.*<dt>location</dt>.*<dd class=monospace>{send_txid}:0:0</dd>.*",
    ),
  );
}

#[test]
fn send_on_mainnnet_refuses_to_work_with_wallet_name_foo() {
  let rpc_server = test_bitcoincore_rpc::builder().wallet_name("foo").build();
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(
    format!("wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw {txid}:0:0")
  )
  .rpc_server(&rpc_server)
  .expected_stderr("error: wallet commands may only be used on mainnet with a wallet named `ord` or whose name starts with `ord-`\n")
  .expected_exit_code(1)
  .run();
}

#[test]
fn send_addresses_must_be_valid_for_network() {
  let rpc_server = test_bitcoincore_rpc::builder().build();
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000)[0].txdata[0].txid();
  create_wallet(&rpc_server);

  CommandBuilder::new(format!(
    "wallet send tord1q497kurvh0fgtedca5angel7j4rdwe0q8h925u0 {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .expected_stderr(
    "error: Address `tord1q497kurvh0fgtedca5angel7j4rdwe0q8h925u0` is not valid for mainnet\n",
  )
  .expected_exit_code(1)
  .run();
}

#[test]
fn send_on_mainnnet_works_with_wallet_named_ord() {
  let rpc_server = test_bitcoincore_rpc::builder().build();
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0].txid();
  create_wallet(&rpc_server);

  let stdout = CommandBuilder::new(format!(
    "wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{}\n", txid), stdout);
}

#[test]
fn send_on_mainnnet_works_with_wallet_whose_name_starts_with_ord() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .wallet_name("ord-foo")
    .build();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!(
    "wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{}\n", txid), stdout);
}

#[test]
fn send_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks_with_subsidy(1, 800)[0].txdata[0].txid();
  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {txid}:0:0 degenerate.png"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
  .run();

  let txid = rpc_server.mine_blocks_with_subsidy(1, 100)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
  .run();
}

#[test]
fn do_not_accidentally_send_an_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();
  let stdout = CommandBuilder::new(format!(
    "wallet inscribe --satpoint {txid}:0:0 degenerate.png"
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
    "wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw  {inscription_utxo}:55"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: cannot send {inscription_utxo}:55 without also sending inscription {inscription_id} at {inscription_utxo}:0\n"
  ))
  .run();
}

#[test]
fn inscriptions_cannot_be_sent_by_satpoint() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let stdout = CommandBuilder::new(format!("wallet inscribe --satpoint {txid}:0:0 hello.txt"))
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  let reveal_txid = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(format!(
    "wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw   {reveal_txid}:0:0"
  ))
  .write("hello.txt", "HELLOWORLD")
  .rpc_server(&rpc_server)
  .expected_stderr("error: inscriptions must be sent by inscription ID\n")
  .expected_exit_code(1)
  .run();
}
#[test]
fn send_btc() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw 1btc")
    .rpc_server(&rpc_server)
    .expected_stdout("0000000000000000000000000000000000000000000000000000000000000000\n")
    .run();

  assert_eq!(
    rpc_server.sent(),
    &[Sent {
      amount: 1.0,
      address: "bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv"
        .parse::<Address>()
        .unwrap(),
      locked: Vec::new(),
    }]
  )
}

#[test]
fn send_btc_locks_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new("wallet inscribe hello.txt")
    .write("hello.txt", "HELLOWORLD")
    .rpc_server(&rpc_server)
    .stdout_regex("commit\t[[:xdigit:]]{64}\nreveal\t[[:xdigit:]]{64}\n")
    .run();

  let inscription_id = reveal_txid_from_inscribe_stdout(&stdout);

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw 1btc")
    .rpc_server(&rpc_server)
    .expected_stdout("0000000000000000000000000000000000000000000000000000000000000000\n")
    .run();

  assert_eq!(
    rpc_server.sent(),
    &[Sent {
      amount: 1.0,
      address: "bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv"
        .parse::<Address>()
        .unwrap(),
      locked: vec![OutPoint {
        txid: inscription_id,
        vout: 0,
      }]
    }]
  )
}

#[test]
fn send_btc_fails_if_lock_unspent_fails() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .fail_lock_unspent(true)
    .build();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet send ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw 1btc")
    .rpc_server(&rpc_server)
    .expected_stderr("error: failed to lock ordinal UTXOs\n")
    .expected_exit_code(1)
    .run();
}

#[test]
fn refuse_to_send_to_cardinal_address_without_cardinal_flag() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(
    "wallet send bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv 1btc",
  )
  .rpc_server(&rpc_server)
  .expected_stderr("error: refusing to send to cardinal adddress, which may be from wallet without sat control; the `--cardinal` flag bypasses this check\n")
  .expected_exit_code(1)
  .run();
}

#[test]
fn allow_send_to_cardinal_address_with_cardinal_flag() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet send --cardinal ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw 1btc")
    .rpc_server(&rpc_server)
    .expected_stdout("0000000000000000000000000000000000000000000000000000000000000000\n")
    .run();

  assert_eq!(
    rpc_server.sent(),
    &[Sent {
      amount: 1.0,
      address: "bc1qcqgs2pps4u4yedfyl5pysdjjncs8et5utseepv"
        .parse()
        .unwrap(),
      locked: Vec::new(),
    }]
  )
}
