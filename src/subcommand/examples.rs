use super::*;

pub(crate) fn run(server: super::server::Server) -> Result {
  let tmpdir = TempDir::new()?;

  let rpc_port = TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();

  let mut bitcoind = Command::new("bitcoind")
    .arg(format!("-datadir={}", tmpdir.path().display()))
    .arg("-regtest")
    .arg("-txindex=1")
    .arg("-reindex=1")
    .arg("-listen=0")
    .arg(format!("-rpcport={rpc_port}"))
    .spawn()?;

  // todo: wait for rpc port to be open
  thread::sleep(Duration::from_secs(1));

  let options = Options {
    chain_argument: Chain::Regtest,
    bitcoin_data_dir: Some(tmpdir.path().into()),
    data_dir: Some(tmpdir.path().into()),
    rpc_url: Some(format!("127.0.0.1:{rpc_port}")),
    index_sats: true,
    ..Options::default()
  };

  super::wallet::create::run(options.clone())?;

  let rpc_client = options.bitcoin_rpc_client()?;

  let address = rpc_client.get_new_address(None, None)?;

  rpc_client.generate_to_address(101, &address)?;

  for result in fs::read_dir("examples")? {
    let entry = result?;

    Arguments {
      options: options.clone(),
      subcommand: Subcommand::Wallet(super::wallet::Wallet::Inscribe(
        super::wallet::inscribe::Inscribe {
          file: entry.path(),
          satpoint: None,
        },
      )),
    }
    .run()?;
  }

  rpc_client.generate_to_address(1, &address)?;

  Arguments {
    options: options.clone(),
    subcommand: Subcommand::Server(server),
  }
  .run()?;

  bitcoind.kill()?;

  Ok(())
}
