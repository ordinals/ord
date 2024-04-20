use super::*;

#[test]
fn bridge_lock() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let _rune = etch(&core, &ord, Rune(RUNE));

  let out = CommandBuilder::new(format!("--chain regtest --index-runes wallet bridge-lock 100:{}", Rune(RUNE)))
    .core(&core)
    .ord(&ord)
    .stdout_regex(".*") // HACK
    .run_and_extract_stdout();

  println!("out: {}", out);
}

#[test]
fn bridge_unlock() {}
