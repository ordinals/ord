use {super::*, crate::wallet::batch, colored::Colorize, std::net::TcpListener};

struct KillOnDrop(process::Child);

impl Drop for KillOnDrop {
  fn drop(&mut self) {
    let _ = Command::new("kill").arg(self.0.id().to_string()).status();

    let _ = self.0.kill();

    let _ = self.0.wait();
  }
}

#[derive(Debug, Parser)]
pub(crate) struct Env {
  #[arg(default_value = "env", help = "Create env in <DIRECTORY>.")]
  directory: PathBuf,
  #[arg(
    long,
    help = "Decompress encoded content. Currently only supports brotli. Be careful using this on production instances. A decompressed inscription may be arbitrarily large, making decompression a DoS vector."
  )]
  pub(crate) decompress: bool,
  #[arg(
    long,
    help = "Proxy `/content/INSCRIPTION_ID` and other recursive endpoints to `<PROXY>` if the inscription is not present on current chain."
  )]
  pub(crate) proxy: Option<Url>,
}

#[derive(Serialize)]
struct Info {
  bitcoin_cli_command: Vec<String>,
  bitcoind_port: u16,
  ord_port: u16,
  ord_wallet_command: Vec<String>,
}

impl Env {
  pub(crate) fn run(self) -> SubcommandResult {
    let bitcoind_port = TcpListener::bind("127.0.0.1:9000")
      .ok()
      .map(|listener| listener.local_addr().unwrap().port());

    let ord_port = TcpListener::bind("127.0.0.1:9001")
      .ok()
      .map(|listener| listener.local_addr().unwrap().port());

    let (bitcoind_port, ord_port) = (
      bitcoind_port.unwrap_or(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port()),
      ord_port.unwrap_or(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port()),
    );

    let relative = self.directory.to_str().unwrap().to_string();
    let absolute = std::env::current_dir()?.join(&self.directory);
    let absolute_str = absolute
      .to_str()
      .with_context(|| format!("directory `{}` is not valid unicode", absolute.display()))?;

    fs::create_dir_all(&absolute)?;

    let bitcoin_conf = absolute.join("bitcoin.conf");

    if !bitcoin_conf.try_exists()? {
      fs::write(
        bitcoin_conf,
        format!(
          "datacarriersize=1000000
regtest=1
datadir={absolute_str}
listen=0
txindex=1
[regtest]
rpcport={bitcoind_port}
",
        ),
      )?;
    }

    fs::write(absolute.join("inscription.txt"), "FOO")?;

    let yaml = serde_yaml::to_string(&batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: "FOO".parse::<SpacedRune>().unwrap(),
        supply: "2000".parse().unwrap(),
        premine: "1000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          amount: "1000".parse().unwrap(),
          cap: 1,
          ..default()
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("env/inscription.txt".into()),
        ..default()
      }],
      ..default()
    })
    .unwrap();

    let batch_yaml = absolute.join("batch.yaml");

    if !batch_yaml.try_exists()? {
      fs::write(absolute.join("batch.yaml"), yaml)?;
    }

    let _bitcoind = KillOnDrop(
      Command::new("bitcoind")
        .arg(format!("-conf={}", absolute.join("bitcoin.conf").display()))
        .stdout(Stdio::null())
        .spawn()
        .expect("failed to start bitcoind"),
    );

    loop {
      if absolute.join("regtest/.cookie").try_exists()? {
        break;
      }
    }

    let rpc_url = format!("http://localhost:{bitcoind_port}");

    let server_url = format!("http://127.0.0.1:{ord_port}");

    let config = absolute.join("ord.yaml");

    if !config.try_exists()? {
      fs::write(
        config,
        serde_yaml::to_string(&Settings::for_env(&absolute, &rpc_url, &server_url))?,
      )?;
    }

    let ord = std::env::current_exe()?;

    let decompress = self.decompress;
    let proxy = self.proxy.map(|url| url.to_string());

    let mut command = Command::new(&ord);
    let ord_server = command
      .arg("--datadir")
      .arg(&absolute)
      .arg("server")
      .arg("--polling-interval=100ms")
      .arg("--http-port")
      .arg(ord_port.to_string());

    if decompress {
      ord_server.arg("--decompress");
    }

    if let Some(proxy) = proxy {
      ord_server.arg("--proxy").arg(proxy);
    }

    let _ord = KillOnDrop(ord_server.spawn()?);

    thread::sleep(Duration::from_millis(250));

    if !absolute.join("regtest/wallets/ord").try_exists()? {
      let status = Command::new(&ord)
        .arg("--datadir")
        .arg(&absolute)
        .arg("wallet")
        .arg("create")
        .status()?;

      ensure!(status.success(), "failed to create wallet: {status}");

      let output = Command::new(&ord)
        .arg("--datadir")
        .arg(&absolute)
        .arg("wallet")
        .arg("receive")
        .output()?;

      ensure!(
        output.status.success(),
        "failed to generate receive address: {status}"
      );

      let receive = serde_json::from_slice::<wallet::receive::Output>(&output.stdout)?;

      let status = Command::new("bitcoin-cli")
        .arg(format!("-datadir={relative}"))
        .arg("generatetoaddress")
        .arg("200")
        .arg(
          receive
            .addresses
            .first()
            .cloned()
            .unwrap()
            .require_network(Network::Regtest)?
            .to_string(),
        )
        .status()?;

      ensure!(status.success(), "failed to create wallet: {status}");
    }

    serde_json::to_writer_pretty(
      File::create(self.directory.join("env.json"))?,
      &Info {
        bitcoind_port,
        ord_port,
        bitcoin_cli_command: vec!["bitcoin-cli".into(), format!("-datadir={relative}")],
        ord_wallet_command: vec![
          ord.to_str().unwrap().into(),
          "--datadir".into(),
          absolute.to_str().unwrap().into(),
          "wallet".into(),
        ],
      },
    )?;

    let datadir = if relative
      .chars()
      .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
      relative
    } else {
      format!("'{relative}'")
    };

    eprintln!(
      "{}
{server_url}
{}
bitcoin-cli -datadir={datadir} getblockchaininfo
{}
{} --datadir {datadir} wallet balance",
      "`ord` server URL:".blue().bold(),
      "Example `bitcoin-cli` command:".blue().bold(),
      "Example `ord` command:".blue().bold(),
      ord.display(),
    );

    loop {
      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break Ok(None);
      }

      thread::sleep(Duration::from_millis(100));
    }
  }
}
