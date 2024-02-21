use {super::*, colored::Colorize, std::net::TcpListener};

struct KillOnDrop(process::Child);

impl Drop for KillOnDrop {
  fn drop(&mut self) {
    assert!(Command::new("kill")
      .arg(self.0.id().to_string())
      .status()
      .unwrap()
      .success());
    self.0.wait().unwrap();
  }
}

#[derive(Debug, Parser)]
pub(crate) struct Env {
  #[arg(default_value = "env", help = "Create env in <DIRECTORY>.")]
  directory: PathBuf,
}

#[derive(Serialize)]
struct Info {
  bitcoind_port: u16,
  ord_port: u16,
  bitcoin_cli_command: Vec<String>,
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

    fs::write(
      absolute.join("bitcoin.conf"),
      format!(
        "regtest=1
datadir={absolute_str}
listen=0
txindex=1
[regtest]
rpcport={bitcoind_port}
",
      ),
    )?;

    let _bitcoind = KillOnDrop(
      Command::new("bitcoind")
        .arg(format!("-conf={}", absolute.join("bitcoin.conf").display()))
        .stdout(Stdio::null())
        .spawn()?,
    );

    loop {
      if absolute.join("regtest/.cookie").try_exists()? {
        break;
      }
    }

    let ord = std::env::current_exe()?;

    let rpc_url = format!("http://localhost:{bitcoind_port}");

    let _ord = KillOnDrop(
      Command::new(&ord)
        .arg("--regtest")
        .arg("--bitcoin-data-dir")
        .arg(&absolute)
        .arg("--data-dir")
        .arg(&absolute)
        .arg("--rpc-url")
        .arg(&rpc_url)
        .arg("server")
        .arg("--polling-interval=100ms")
        .arg("--http-port")
        .arg(ord_port.to_string())
        .spawn()?,
    );

    thread::sleep(Duration::from_millis(250));

    let server_url = format!("http://127.0.0.1:{ord_port}");

    if !absolute.join("regtest/wallets/ord").try_exists()? {
      let status = Command::new(&ord)
        .arg("--regtest")
        .arg("--bitcoin-data-dir")
        .arg(&absolute)
        .arg("--data-dir")
        .arg(&absolute)
        .arg("--rpc-url")
        .arg(&rpc_url)
        .arg("wallet")
        .arg("create")
        .status()?;

      ensure!(status.success(), "failed to create wallet: {status}");

      let output = Command::new(&ord)
        .arg("--regtest")
        .arg("--bitcoin-data-dir")
        .arg(&absolute)
        .arg("--data-dir")
        .arg(&absolute)
        .arg("--rpc-url")
        .arg(&rpc_url)
        .arg("wallet")
        .arg("--server-url")
        .arg(&server_url)
        .arg("receive")
        .output()?;

      ensure!(
        output.status.success(),
        "failed to generate receive address: {status}"
      );

      let receive =
        serde_json::from_slice::<crate::subcommand::wallet::receive::Output>(&output.stdout)?;

      let address = receive.address.require_network(Network::Regtest)?;

      let status = Command::new("bitcoin-cli")
        .arg(format!("-datadir={relative}"))
        .arg("generatetoaddress")
        .arg("200")
        .arg(address.to_string())
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
          "--regtest".into(),
          "--bitcoin-data-dir".into(),
          relative.clone(),
          "--data-dir".into(),
          relative.clone(),
          "--rpc-url".into(),
          rpc_url.clone(),
          "wallet".into(),
          "--server-url".into(),
          server_url.clone(),
        ],
      },
    )?;

    eprintln!(
      "{}
bitcoin-cli -datadir='{relative}' getblockchaininfo
{}
{} --regtest --bitcoin-data-dir '{relative}' --data-dir '{relative}' --rpc-url '{}' wallet --server-url  {} balance",
      "Example `bitcoin-cli` command:".blue().bold(),
      "Example `ord` command:".blue().bold(),
      ord.display(),
      rpc_url,
      server_url,
    );

    loop {
      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break Ok(None);
      }

      thread::sleep(Duration::from_millis(100));
    }
  }
}
