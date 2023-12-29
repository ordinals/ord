use {super::*, fee_rate::FeeRate, std::sync::atomic};

#[derive(Debug, Parser)]
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

    let rpc_port = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();

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
        .arg(format!("-rpcport={rpc_port}"))
        .spawn()
        .context("failed to spawn `bitcoind`")?,
    );

    let options = Options {
      chain_argument: Chain::Regtest,
      bitcoin_data_dir: Some(bitcoin_data_dir),
      data_dir: tmpdir.path().into(),
      rpc_url: Some(format!("127.0.0.1:{rpc_port}")),
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

    super::wallet::create::Create {
      passphrase: "".into(),
    }
    .run("ord".into(), options.clone())?;

    let rpc_client = options.bitcoin_rpc_client(None)?;

    let address = rpc_client
      .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?
      .require_network(Network::Regtest)?;

    eprintln!("Mining blocks…");

    rpc_client.generate_to_address(101, &address)?;

    if let Some(files) = self.files {
      for file in files {
        Arguments {
          options: options.clone(),
          subcommand: Subcommand::Wallet(super::wallet::Wallet {
            name: "ord".into(),
            subcommand: super::wallet::Subcommand::Inscribe(super::wallet::inscribe::Inscribe {
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
              postage: Some(TARGET_POSTAGE),
              reinscribe: false,
              satpoint: None,
              sat: None,
            }),
          }),
        }
        .run()?;

        rpc_client.generate_to_address(1, &address)?;
      }
    }

    if let Some(batches) = self.batches {
      for batch in batches {
        Arguments {
          options: options.clone(),
          subcommand: Subcommand::Wallet(super::wallet::Wallet {
            name: "ord".into(),
            subcommand: super::wallet::Subcommand::Inscribe(super::wallet::inscribe::Inscribe {
              batch: Some(batch),
              cbor_metadata: None,
              commit_fee_rate: None,
              compress: false,
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
              satpoint: None,
              sat: None,
            }),
          }),
        }
        .run()?;

        rpc_client.generate_to_address(1, &address)?;
      }
    }

    if let Some(blocktime) = self.blocktime {
      eprintln!(
        "Mining blocks every {}...",
        "second".tally(blocktime.try_into().unwrap())
      );

      let running = Arc::new(AtomicBool::new(true));

      let handle = {
        let running = running.clone();

        std::thread::spawn(move || {
          while running.load(atomic::Ordering::SeqCst) {
            rpc_client.generate_to_address(1, &address).unwrap();
            thread::sleep(Duration::from_secs(blocktime));
          }
        })
      };

      Arguments {
        options,
        subcommand: Subcommand::Server(self.server),
      }
      .run()?;

      running.store(false, atomic::Ordering::SeqCst);

      handle.join().unwrap();
    } else {
      Arguments {
        options,
        subcommand: Subcommand::Server(self.server),
      }
      .run()?;
    }

    Ok(Box::new(Empty {}))
  }
}
