use super::*;

#[test]
fn bridge_lock() {
  let core = mockcore::builder().network(Network::Regtest).build();

  // start ord.
  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  // mine a block to create funds for the test from the subsidy.
  core.mine_blocks(1);

  let rune = Rune(RUNE);

  // create an rune to bridge.
  let _etched = etch(&core, &ord, rune);

  // bridge the rune out.
  let output = CommandBuilder::new(format!(
    "--chain regtest --index-runes wallet bridge-lock --fee-rate 1 100:{}",
    rune
  ))
  .core(&core)
  .ord(&ord)
  .stdout_regex(".*") // HACK: show all output.
  .run_and_extract_stdout();

  print!("output: {}", output);
}

#[test]
fn bridge_unlock() {}
