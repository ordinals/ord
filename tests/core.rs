use bitcoin::Address;

use {
  super::*,
  bitcoincore_rpc::{Client, RpcApi},
  std::ffi::OsString,
};

struct KillOnDrop(std::process::Child);

impl Drop for KillOnDrop {
  fn drop(&mut self) {
    assert!(Command::new("kill")
      .arg(self.0.id().to_string())
      .status()
      .unwrap()
      .success());
  }
}

#[test]
#[ignore]
fn preview() {
  let port = TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();

  let examples = fs::read_dir("examples")
    .unwrap()
    .map(|entry| {
      entry
        .unwrap()
        .path()
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .into()
    })
    .filter(|example| example != "examples/av1.mp4")
    .collect::<Vec<String>>();

  let mut args = vec![
    "preview".to_string(),
    "--http-port".to_string(),
    port.to_string(),
  ];
  args.extend(examples.clone());

  let builder = CommandBuilder::new(args);

  let _child = KillOnDrop(builder.command().spawn().unwrap());

  for attempt in 0.. {
    if let Ok(response) = reqwest::blocking::get(format!("http://127.0.0.1:{port}/status")) {
      if response.status() == 200 {
        assert_eq!(response.text().unwrap(), "OK");
        break;
      }
    }

    if attempt == 100 {
      panic!("Server did not respond to status check",);
    }

    thread::sleep(Duration::from_millis(500));
  }

  assert_regex_match!(
    reqwest::blocking::get(format!("http://127.0.0.1:{port}/inscriptions"))
      .unwrap()
      .text()
      .unwrap(),
    format!(".*(<a href=/inscription/.*){{{}}}.*", examples.len())
  );
}

fn get_free_port() -> u16 {
  TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port()
}

fn ord(
  cookiefile: &std::path::Path,
  ord_data_dir: &std::path::Path,
  rpc_port: u16,
  args: &[&str],
) -> Result<String, String> {
  let mut ord = Command::new(executable_path("ord"));

  ord
    .env("ORD_INTEGRATION_TEST", "1")
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .current_dir(ord_data_dir)
    .arg("--regtest")
    .arg("--data-dir")
    .arg(ord_data_dir)
    .arg("--rpc-url")
    .arg(&format!("127.0.0.1:{}", rpc_port))
    .arg("--cookie-file")
    .arg(cookiefile.to_str().unwrap())
    .args(args);

  let output = ord.output().unwrap();

  if output.status.success() {
    Ok(String::from(str::from_utf8(&output.stdout).unwrap()))
  } else {
    Err(String::from(str::from_utf8(&output.stderr).unwrap()))
  }
}

#[test]
#[ignore]
fn inscribe_child() {
  let rpc_port = get_free_port();

  let tmp_dir_1 = TempDir::new().unwrap();
  let bitcoin_data_dir = tmp_dir_1.path().join("bitcoin");
  fs::create_dir(&bitcoin_data_dir).unwrap();

  let tmp_dir_2 = TempDir::new().unwrap();
  let ord_data_dir = tmp_dir_2.path().join("ord");
  fs::create_dir(&ord_data_dir).unwrap();

  let _bitcoind = KillOnDrop(
    Command::new("bitcoind")
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .arg({
        let mut arg = OsString::from("-datadir=");
        arg.push(&bitcoin_data_dir);
        arg
      })
      .arg("-regtest")
      .arg("-txindex")
      .arg("-listen=0")
      .arg("-minrelaytxfee=0")
      .arg(format!("-rpcport={rpc_port}"))
      .spawn()
      .expect("failed to spawn `bitcoind`"),
  );

  let cookiefile = bitcoin_data_dir.as_path().join("regtest/.cookie");

  for attempt in 0.. {
    if Client::new(
      &format!("127.0.0.1:{rpc_port}"),
      bitcoincore_rpc::Auth::CookieFile(cookiefile.clone()),
    )
    .is_ok()
    {
      break;
    }

    if attempt == 500 {
      panic!("Bitcoin Core RPC did not respond");
    }

    thread::sleep(Duration::from_millis(50));
  }

  let _ = ord(&cookiefile, &ord_data_dir, rpc_port, &["wallet", "create"]);

  // get funds in wallet
  // inscribe parent
  // mine block
  // inscribe child with parent

  let rpc_client = Client::new(
    &format!("127.0.0.1:{rpc_port}/wallet/ord"),
    bitcoincore_rpc::Auth::CookieFile(cookiefile.clone()),
  )
  .unwrap();

  let address = rpc_client
    .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))
    .unwrap();

  let not_ours = Address::from_str("bcrt1qyr2zc4lhadk9k35hwfh2unn7hgvtpwpx8mjx4h").unwrap();

  rpc_client.generate_to_address(1, &address).unwrap();
  rpc_client.generate_to_address(100, &not_ours).unwrap(); // need to mine 100 blocks for coins to become spendable. use address outside our wallet to prevent slow rescan

  fs::write(ord_data_dir.as_path().join("parent.txt"), "Pater").unwrap();

  #[derive(Deserialize, Debug)]
  #[allow(dead_code)] // required because of the `serde` macro, can't use _
  struct Output {
    commit: String,
    inscription: String,
    parent: Option<String>,
    reveal: String,
    fees: u64,
  }

  let output: Output = match ord(
    &cookiefile,
    &ord_data_dir,
    rpc_port,
    &["wallet", "inscribe", "parent.txt"],
  ) {
    Ok(s) => serde_json::from_str(&s)
      .unwrap_or_else(|err| panic!("Failed to deserialize JSON: {err}\n{s}")),
    Err(e) => panic!("error inscribing parent: {}", e),
  };
  let parent_id = output.inscription;

  rpc_client.generate_to_address(1, &address).unwrap();
  thread::sleep(Duration::from_secs(1));

  fs::write(ord_data_dir.as_path().join("child.txt"), "Filius").unwrap();
  let output: Output = match ord(
    &cookiefile,
    &ord_data_dir,
    rpc_port,
    &[
      "wallet",
      "inscribe",
      "--fee-rate",
      "2",
      "--parent",
      &parent_id,
      "child.txt",
    ],
  ) {
    Ok(s) => serde_json::from_str(&s)
      .unwrap_or_else(|err| panic!("Failed to deserialize JSON: {err}\n{s}")),
    Err(e) => panic!("error inscribing child with parent: {}", e),
  };

  let child_id = output.inscription;

  rpc_client.generate_to_address(1, &address).unwrap();

  let ord_port = 8080;
  let _ord_server = KillOnDrop(
    Command::new(executable_path("ord"))
      .env("ORD_INTEGRATION_TEST", "1")
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .current_dir(ord_data_dir.clone())
      .arg("--regtest")
      .arg("--data-dir")
      .arg(ord_data_dir.as_path())
      .arg("--rpc-url")
      .arg(&format!("127.0.0.1:{}", rpc_port))
      .arg("--cookie-file")
      .arg(cookiefile.to_str().unwrap())
      .arg("server")
      .arg("--http-port")
      .arg(&format!("{ord_port}"))
      .spawn()
      .expect("failed to spawn `ord server`"),
  );

  let client = reqwest::blocking::Client::builder()
    .redirect(reqwest::redirect::Policy::none())
    .build()
    .unwrap();

  for i in 0.. {
    match client
      .get(format!("http://127.0.0.1:{ord_port}/status"))
      .send()
    {
      Ok(_) => break,
      Err(err) => {
        if i == 400 {
          panic!("server failed to start: {err}");
        }
      }
    }

    thread::sleep(Duration::from_millis(25));
  }

  let response = client
    .get(format!(
      "http://127.0.0.1:{ord_port}/inscription/{parent_id}"
    ))
    .send()
    .unwrap();

  assert_regex_match!(response.text().unwrap(), &format!(".*id.*{}.*", parent_id));

  thread::sleep(Duration::from_secs(10));

  let response = client
    .get(format!(
      "http://127.0.0.1:{ord_port}/inscription/{child_id}"
    ))
    .send()
    .unwrap();

  assert_regex_match!(
    response.text().unwrap(),
    &format!(".*parent.*{}.*", parent_id)
  );
}
