use {super::*, fee_rate::FeeRate, reqwest::Url, std::sync::atomic};

#[derive(Debug, Parser, Clone)]
pub(crate) struct Preview {
  #[command(flatten)]
  server: super::server::Server,
  #[arg(
    num_args = 0..,
    long,
    help = "Inscribe inscriptions defined in <BATCHES>."
  )]
  batches: Option<Vec<PathBuf>>,
  #[arg(long, help = "Automatically mine a block every <BLOCKTIME> seconds.")]
  blocktime: Option<u64>,
  #[arg(num_args = 0.., long, help = "Inscribe contents of <FILES>.")]
  files: Option<Vec<PathBuf>>,
}

#[derive(Debug, Parser)]
pub(crate) struct Batch {
  batch_files: Vec<PathBuf>,
}

#[derive(Debug, Parser)]
pub(crate) struct File {
  files: Vec<PathBuf>,
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

    let bitcoin_rpc_port = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();

    let bitcoin_data_dir = tmpdir.path().join("bitcoin");

    fs::create_dir(&bitcoin_data_dir)?;

    eprintln!("Spawning bitcoind…");

    let _bitcoind = KillOnDrop(
      Command::new("bitcoind")
        .arg({
          let mut arg = OsString::from("-datadir=");
          arg.push(&bitcoin_data_dir);
          arg
        })
        .arg("-listen=0")
        .arg("-printtoconsole=0")
        .arg("-regtest")
        .arg("-txindex")
        .arg(format!("-rpcport={bitcoin_rpc_port}"))
        .spawn()
        .context("failed to spawn `bitcoind`")?,
    );

    let options = Options {
      chain_argument: Chain::Regtest,
      bitcoin_data_dir: Some(bitcoin_data_dir.clone()),
      data_dir: tmpdir.path().into(),
      rpc_url: Some(format!("http://127.0.0.1:{bitcoin_rpc_port}")),
      index_sats: true,
      ..Options::default()
    };

    for attempt in 0.. {
      if options.bitcoin_rpc_client(None).is_ok() {
        break;
      }

      if attempt == 100 {
        panic!("Bitcoin Core RPC did not respond");
      }

      thread::sleep(Duration::from_millis(50));
    }

    let mut ord_server_args = self.server.clone();

    ord_server_args.enable_json_api = true;

    let ord_server_url: Url = format!(
      "http://127.0.0.1:{}",
      ord_server_args.http_port.unwrap_or(8080)
    )
    .parse()
    .unwrap();

    Arguments {
      options: options.clone(),
      subcommand: Subcommand::Wallet(crate::subcommand::wallet::WalletCommand {
        name: "ord".into(),
        no_sync: false,
        server_url: ord_server_url.clone(),
        subcommand: crate::subcommand::wallet::Subcommand::Create(
          crate::subcommand::wallet::create::Create {
            passphrase: "".into(),
          },
        ),
      }),
    }
    .run()
    .unwrap();

    let bitcoin_rpc_client = options.bitcoin_rpc_client(None)?;

    let address = bitcoin_rpc_client
      .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?
      .require_network(Network::Regtest)?;

    eprintln!("Mining blocks…");

    bitcoin_rpc_client.generate_to_address(101, &address)?;

    let running = Arc::new(AtomicBool::new(true));

    let mining_handle = if let Some(blocktime) = self.blocktime {
      let bitcoin_rpc_client = options.bitcoin_rpc_client(None)?;
      let address = address.clone();
      let running = running.clone();

      eprintln!(
        "Mining blocks every {}...",
        "second".tally(blocktime.try_into().unwrap())
      );

      Some(std::thread::spawn(move || {
        while running.load(atomic::Ordering::SeqCst) {
          bitcoin_rpc_client.generate_to_address(1, &address).unwrap();
          thread::sleep(Duration::from_secs(blocktime));
        }
      }))
    } else {
      None
    };

    let ord_server_handle = {
      let options = options.clone();

      std::thread::spawn(move || {
        Arguments {
          options,
          subcommand: Subcommand::Server(ord_server_args),
        }
        .run()
        .unwrap()
      })
    };

    if let Some(files) = self.files {
      for file in files {
        Arguments {
          options: options.clone(),
          subcommand: Subcommand::Wallet(super::wallet::WalletCommand {
            name: "ord".into(),
            no_sync: false,
            server_url: ord_server_url.clone(),
            subcommand: super::wallet::Subcommand::Inscribe(super::wallet::inscribe::Inscribe {
              batch: None,
              cbor_metadata: None,
              commit_fee_rate: None,
              compress: false,
              delegate: None,
              destination: None,
              dry_run: false,
              fee_rate: FeeRate::try_from(1.0).unwrap(),
              file: Some(file),
              json_metadata: None,
              metaprotocol: None,
              no_backup: true,
              no_limit: false,
              parent: None,
              postage: Some(TARGET_POSTAGE),
              reinscribe: false,
              sat: None,
              satpoint: None,
            }),
          }),
        }
        .run()?;

        bitcoin_rpc_client.generate_to_address(1, &address)?;
      }
    }

    println!("hereasldfkjasdflkjasdf");

    if let Some(batches) = self.batches {
      for batch in batches {
        Arguments {
          options: options.clone(),
          subcommand: Subcommand::Wallet(super::wallet::WalletCommand {
            name: "ord".into(),
            no_sync: false,
            server_url: ord_server_url.clone(),
            subcommand: super::wallet::Subcommand::Inscribe(super::wallet::inscribe::Inscribe {
              batch: Some(batch),
              cbor_metadata: None,
              commit_fee_rate: None,
              compress: false,
              delegate: None,
              destination: None,
              dry_run: false,
              fee_rate: FeeRate::try_from(1.0).unwrap(),
              file: None,
              json_metadata: None,
              metaprotocol: None,
              no_backup: true,
              no_limit: false,
              parent: None,
              postage: Some(TARGET_POSTAGE),
              reinscribe: false,
              sat: None,
              satpoint: None,
            }),
          }),
        }
        .run()?;

        bitcoin_rpc_client.generate_to_address(1, &address)?;
      }
    }

    ord_server_handle.join().unwrap();

    running.store(false, atomic::Ordering::SeqCst);

    if let Some(mining_handle) = mining_handle {
      mining_handle.join().unwrap();
    }

    Ok(None)
  }
}
