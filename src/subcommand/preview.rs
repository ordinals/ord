use {super::*, fee_rate::FeeRate};

#[derive(Debug, Parser)]
pub(crate) struct Preview {
  #[command(flatten)]
  server: super::server::Server,
  inscriptions: Vec<PathBuf>,
}

struct KillOnDrop(process::Child);

impl Drop for KillOnDrop {
  fn drop(&mut self) {
    self.0.kill().unwrap()
  }
}

impl Preview {
  pub(crate) fn run(self) -> SubcommandResult {
    let tmpdir = TempDir::new()?;

    let rpc_port = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();

    let bitcoin_data_dir = tmpdir.path().join("bitcoin");

    fs::create_dir(&bitcoin_data_dir)?;

    let _bitcoind = KillOnDrop(
      Command::new("bitcoind")
        .arg({
          let mut arg = OsString::from("-datadir=");
          arg.push(&bitcoin_data_dir);
          arg
        })
        .arg("-regtest")
        .arg("-txindex")
        .arg("-listen=0")
        .arg(format!("-rpcport={rpc_port}"))
        .spawn()
        .context("failed to spawn `bitcoind`")?,
    );

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

    super::wallet::Wallet::Create(super::wallet::create::Create {
      passphrase: "".into(),
    })
    .run(options.clone())?;

    let rpc_client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let address = rpc_client
      .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?
      .require_network(Network::Regtest)?;

    rpc_client.generate_to_address(101, &address)?;

    for file in self.inscriptions {
      Arguments {
        options: options.clone(),
        subcommand: Subcommand::Wallet(super::wallet::Wallet::Inscribe(
          super::wallet::inscribe::Inscribe {
            batch: None,
            cbor_metadata: None,
            commit_fee_rate: None,
            compress: false,
            destination: None,
            dry_run: false,
            fee_rate: FeeRate::try_from(1.0).unwrap(),
            file: Some(file),
            json_metadata: None,
            metaprotocol: None,
            no_backup: true,
            no_limit: false,
            parent: None,
            postage: Some(TransactionBuilder::TARGET_POSTAGE),
            reinscribe: false,
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
    .run()
  }
}
