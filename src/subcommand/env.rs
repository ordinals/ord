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

impl Env {
  pub(crate) fn run(self) -> SubcommandResult {
    let (bitcoind_port, ord_port) = (
      TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port(),
      TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port(),
    );

    let env = std::env::current_dir()?.join(&self.directory);

    fs::create_dir_all(&env)?;

    let env_string = env
      .to_str()
      .with_context(|| format!("directory `{}` is not valid unicode", env.display()))?;

    let config = env.join("bitcoin.conf").to_str().unwrap().to_string();

    fs::write(
      env.join("bitcoin.conf"),
      format!(
        "regtest=1
datadir={env_string}
listen=0
txindex=1
[regtest]
rpcport={bitcoind_port}
",
      ),
    )?;

    let _bitcoind = KillOnDrop(
      Command::new("bitcoind")
        .arg(format!("-conf={config}"))
        .stdout(Stdio::null())
        .spawn()?,
    );

    loop {
      if env.join("regtest/.cookie").try_exists()? {
        break;
      }
    }

    let ord = std::env::current_exe()?;

    let rpc_url = format!("http://localhost:{bitcoind_port}");

    let _ord = KillOnDrop(
      Command::new(&ord)
        .arg("--regtest")
        .arg("--bitcoin-data-dir")
        .arg(&env)
        .arg("--data-dir")
        .arg(&env)
        .arg("--rpc-url")
        .arg(&rpc_url)
        .arg("server")
        .arg("--http-port")
        .arg(ord_port.to_string())
        .spawn()?,
    );

    thread::sleep(Duration::from_millis(250));

    if !env.join("regtest/wallets/ord").try_exists()? {
      let status = Command::new(&ord)
        .arg("--regtest")
        .arg("--bitcoin-data-dir")
        .arg(&env)
        .arg("--data-dir")
        .arg(&env)
        .arg("--rpc-url")
        .arg(&rpc_url)
        .arg("wallet")
        .arg("create")
        .status()?;

      ensure!(status.success(), "failed to create wallet: {status}");
    }

    let directory = self.directory.to_str().unwrap().to_string();

    eprintln!(
      "{}
bitcoin-cli -datadir='{directory}' getblockchaininfo
{}
{} --regtest --bitcoin-data-dir '{directory}' --data-dir '{directory}' --rpc-url '{}' wallet --server-url http://127.0.0.1:{ord_port} balance",
      "Example `bitcoin-cli` command:".blue().bold(),
      "Example `ord` command:".blue().bold(),
      ord.display(),
      rpc_url,
    );

    loop {
      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break Ok(None);
      }

      thread::sleep(Duration::from_millis(100));
    }
  }
}
