use {super::*, std::net::TcpListener};

#[test]
fn bridge_lock() {
  // create a listener to mock the universe.
  let listener = TcpListener::bind("127.0.0.1:0").unwrap();
  let listener_address = listener.local_addr().unwrap();

  let core = mockcore::builder().network(Network::Regtest).build();

  // start ord.
  let ord = TestServer::spawn_with_server_args(
    &core,
    &[
      "--regtest",
      "--index-runes",
      &format!("--universe-url=127.0.0.1:{}", listener_address.port()),
    ],
    &[],
  );

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

  // mine the block with the bridge in it.
  core.mine_blocks(1);

  // wait until we have an incoming connection, this will be the proof insert to the universe.
  // TODO: we should validate the proof here.
  for _ in listener.incoming() {
    break;
  }
}

#[test]
fn bridge_unlock() {}
