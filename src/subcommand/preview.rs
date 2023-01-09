use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Preview {
  #[clap(flatten)]
  server: super::server::Server,
  inscriptions: Vec<PathBuf>,
}

impl Preview {
  pub(crate) fn run(self) -> Result {
    let tmpdir = TempDir::new()?;

    let rpc_port = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();

    let bitcoin_data_dir = tmpdir.path().join("bitcoin");

    fs::create_dir(&bitcoin_data_dir)?;

    let mut bitcoind = Command::new("bitcoind")
      .arg({
        let mut arg = OsString::from("-datadir=");
        arg.push(&bitcoin_data_dir);
        arg
      })
      .arg("-regtest")
      .arg("-txindex=1")
      .arg("-listen=0")
      .arg(format!("-rpcport={rpc_port}"))
      .spawn()?;

    let options = Options {
      chain_argument: Chain::Regtest,
      bitcoin_data_dir: Some(bitcoin_data_dir),
      data_dir: Some(tmpdir.path().into()),
      rpc_url: Some(format!("127.0.0.1:{rpc_port}")),
      index_sats: true,
      ..Options::default()
    };

    for attempt in 0.. {
      if options.bitcoin_rpc_client().is_ok() {
        break;
      }

      if attempt == 100 {
        panic!("Bitcoin Core RPC did not respond");
      }

      thread::sleep(Duration::from_millis(50));
    }

    let rpc_client = options.bitcoin_rpc_client()?;

    super::wallet::create::Create::run(
      &super::wallet::create::Create { name: "ord".into() },
      options.clone(),
    )?;

    let address = rpc_client.get_new_address(None, None)?;

    rpc_client.generate_to_address(101, &address)?;

    for file in self.inscriptions {
      Arguments {
        options: options.clone(),
        subcommand: Subcommand::Wallet(super::wallet::Wallet::Inscribe(
          super::wallet::inscribe::Inscribe {
            file,
            no_backup: true,
            satpoint: None,
          },
        )),
      }
      .run()?;

      rpc_client.generate_to_address(1, &address)?;
    }

    rpc_client.generate_to_address(1, &address)?;

    Arguments {
      options,
      subcommand: Subcommand::Server(self.server),
    }
    .run()?;

    bitcoind.kill()?;

    Ok(())
  }
}
