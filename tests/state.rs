use super::*;

fn free_port() -> u16 {
  TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port()
}

pub(crate) struct State {
  bitcoind: Child,
  pub(crate) tempdir: TempDir,
  pub(crate) client: Client,
  pub(crate) wallet: Wallet<MemoryDatabase>,
  pub(crate) blockchain: RpcBlockchain,
  pub(crate) bitcoind_rpc_port: u16,
  ord_http_port: Option<u16>,
  ord: Option<Child>,
  https: bool,
}

static ONCE: Once = Once::new();

impl State {
  pub(crate) fn new() -> Self {
    Self::https(false)
  }

  pub(crate) fn https(https: bool) -> Self {
    ONCE.call_once(env_logger::init);

    let tempdir = TempDir::new().unwrap();

    fs::create_dir(tempdir.path().join("bitcoin")).unwrap();

    let bitcoind_rpc_port = free_port();

    let bitcoind = Command::new("bitcoind")
      .stdout(if log::max_level() >= LevelFilter::Info {
        Stdio::inherit()
      } else {
        Stdio::null()
      })
      .args(&[
        "-txindex=1",
        "-minrelaytxfee=0",
        "-blockmintxfee=0",
        "-dustrelayfee=0",
        "-maxtxfee=21000000",
        "-datadir=bitcoin",
        "-regtest",
        "-networkactive=0",
        "-listen=0",
        &format!("-rpcport={bitcoind_rpc_port}"),
      ])
      .current_dir(&tempdir.path())
      .spawn()
      .unwrap();

    let cookiefile = tempdir.path().join("bitcoin/regtest/.cookie");

    while !cookiefile.is_file() {}

    let client = Client::new(
      &format!("localhost:{bitcoind_rpc_port}"),
      bitcoincore_rpc::Auth::CookieFile(cookiefile.clone()),
    )
    .unwrap();

    for attempt in 0..=300 {
      match client.get_blockchain_info() {
        Ok(_) => break,
        Err(err) => {
          if attempt == 300 {
            panic!("Failed to connect to bitcoind: {err}");
          }
        }
      }
      sleep(Duration::from_millis(100));
    }

    let wallet = Wallet::new(
      Bip84(
        (
          Mnemonic::parse("book fit fly ketchup also elevator scout mind edit fatal where rookie")
            .unwrap(),
          None,
        ),
        KeychainKind::External,
      ),
      None,
      Network::Regtest,
      MemoryDatabase::new(),
    )
    .unwrap();

    let blockchain = RpcBlockchain::from_config(&RpcConfig {
      url: format!("localhost:{bitcoind_rpc_port}"),
      auth: bdk::blockchain::rpc::Auth::Cookie { file: cookiefile },
      network: Network::Regtest,
      wallet_name: "test".to_string(),
      sync_params: None,
    })
    .unwrap();

    State {
      tempdir,
      bitcoind_rpc_port,
      ord_http_port: None,
      bitcoind,
      client,
      wallet,
      ord: None,
      blockchain,
      https,
    }
  }

  pub(crate) fn get_block(&self, height: u64) -> Block {
    self
      .client
      .get_block(&self.client.get_block_hash(height).unwrap())
      .unwrap()
  }

  pub(crate) fn sync(&self) {
    self
      .wallet
      .sync(&self.blockchain, SyncOptions::default())
      .unwrap();
  }

  pub(crate) fn blocks(&self, n: u64) -> Vec<bitcoin::BlockHash> {
    self
      .client
      .generate_to_address(
        n,
        &self
          .wallet
          .get_address(AddressIndex::Peek(0))
          .unwrap()
          .address,
      )
      .unwrap()
  }

  pub(crate) fn transaction(&self, options: TransactionOptions) -> Transaction {
    self.sync();

    let input_value = options
      .slots
      .iter()
      .map(|slot| self.get_block(slot.0 as u64).txdata[slot.1].output[slot.2].value)
      .sum::<u64>();

    let output_value = input_value - options.fee;

    let (mut psbt, _) = {
      let mut builder = self.wallet.build_tx();

      builder
        .manually_selected_only()
        .fee_absolute(options.fee)
        .allow_dust(true)
        .add_utxos(
          &options
            .slots
            .iter()
            .map(|slot| OutPoint {
              txid: self.get_block(slot.0 as u64).txdata[slot.1].txid(),
              vout: slot.2 as u32,
            })
            .collect::<Vec<OutPoint>>(),
        )
        .unwrap()
        .set_recipients(vec![
          (
            options.recipient.unwrap_or_else(|| self
              .wallet
              .get_address(AddressIndex::Peek(0))
              .unwrap()
              .address
              .script_pubkey()),
            output_value / options.output_count as u64
          );
          options.output_count
        ]);

      builder.finish().unwrap()
    };

    if !self.wallet.sign(&mut psbt, SignOptions::default()).unwrap() {
      panic!("Failed to sign transaction");
    }

    let tx = psbt.extract_tx();

    self
      .client
      .call::<Txid>(
        "sendrawtransaction",
        &[tx.raw_hex().into(), 21000000.into()],
      )
      .unwrap();

    tx
  }

  pub(crate) fn request(&mut self, path: &str, status: u16, expected_response: &str) {
    self.request_expected(path, status, Expected::String(expected_response.into()));
  }

  pub(crate) fn request_regex(&mut self, path: &str, status: u16, expected_response: &str) {
    self.request_expected(path, status, Expected::regex(expected_response));
  }

  pub(crate) fn request_expected(&mut self, path: &str, status: u16, expected: Expected) {
    let client = self.http_client();

    if self.ord_http_port.is_none() {
      let ord_http_port = free_port();

      let ord = Command::new(executable_path("ord"))
        .current_dir(self.tempdir.path())
        .env("HOME", self.tempdir.path())
        .arg(format!("--rpc-url=localhost:{}", self.bitcoind_rpc_port))
        .arg("--cookie-file=bitcoin/regtest/.cookie")
        .args([
          "server",
          "--address",
          "127.0.0.1",
          if self.https {
            "--https-port"
          } else {
            "--http-port"
          },
          &ord_http_port.to_string(),
          "--acme-domain",
          "localhost",
        ])
        .spawn()
        .unwrap();

      for attempt in 0..=300 {
        match client
          .get(&format!("https://127.0.0.1:{ord_http_port}/status"))
          .send()
        {
          Ok(response) if response.status().is_success() => break,
          result => {
            if attempt == 300 {
              panic!("Failed to connect to ord server: {result:?}");
            }
          }
        }
        sleep(Duration::from_millis(100));
      }

      self.ord = Some(ord);
      self.ord_http_port = Some(ord_http_port);
    }

    for attempt in 0..=300 {
      let best_hash = self.client.get_best_block_hash().unwrap();
      let bitcoind_height = self
        .client
        .get_block_header_info(&best_hash)
        .unwrap()
        .height as u64;

      let ord_height = client
        .get(&format!(
          "https://127.0.0.1:{}/height",
          self.ord_http_port.unwrap()
        ))
        .send()
        .unwrap()
        .text()
        .unwrap()
        .parse::<u64>()
        .unwrap();

      if ord_height == bitcoind_height {
        break;
      } else {
        if attempt == 300 {
          panic!("Ord height {ord_height} did not catch up to bitcoind height {bitcoind_height}");
        }

        sleep(Duration::from_millis(100));
      }
    }

    let response = client
      .get(&format!(
        "https://127.0.0.1:{}/{}",
        self.ord_http_port.unwrap(),
        path
      ))
      .send()
      .unwrap();

    log::info!("{:?}", response);

    assert_eq!(response.status().as_u16(), status);

    expected.assert_match(&response.text().unwrap());
  }

  pub(crate) fn ord_data_dir(&self) -> PathBuf {
    self
      .tempdir
      .path()
      .join(if cfg!(target_os = "macos") {
        "Library/Application Support/"
      } else {
        ".local/share"
      })
      .join("ord")
  }

  pub(crate) fn http_client(&self) -> reqwest::blocking::Client {
    if self.https {
      let root_certificate = self.create_acme_cache();
      reqwest::blocking::ClientBuilder::new()
        .add_root_certificate(root_certificate)
        .build()
        .unwrap()
    } else {
      reqwest::blocking::ClientBuilder::new().build().unwrap()
    }
  }

  pub(crate) fn create_acme_cache(&self) -> reqwest::Certificate {
    use rcgen::{
      BasicConstraints, Certificate, CertificateParams, IsCa, KeyPair, SanType,
      PKCS_ECDSA_P256_SHA256,
    };
    let root_certificate = {
      let mut params: CertificateParams = Default::default();
      params.key_pair = Some(KeyPair::generate(&PKCS_ECDSA_P256_SHA256).unwrap());
      params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
      Certificate::from_params(params).unwrap()
    };

    let certificate_keys = KeyPair::generate(&PKCS_ECDSA_P256_SHA256).unwrap();
    let certificate_keys_pem = certificate_keys.serialize_pem();
    let certificate = {
      let mut params = CertificateParams::from_ca_cert_pem(
        &root_certificate.serialize_pem().unwrap(),
        certificate_keys,
      )
      .unwrap();
      params
        .subject_alt_names
        .push(SanType::DnsName("localhost".to_string()));
      Certificate::from_params(params).unwrap()
    };
    let certificate_file = vec![
      certificate_keys_pem,
      certificate
        .serialize_pem_with_signer(&root_certificate)
        .unwrap(),
      root_certificate.serialize_pem().unwrap(),
    ]
    .join("\r\n");

    let acme_cache = self.ord_data_dir().join("acme-cache");

    fs::create_dir_all(&acme_cache).unwrap();

    fs::write(
      acme_cache.join("cached_cert_83kei_h4oopqh8sXFFlhGeQJIS_pkJJv-y5XDpnLtyw"),
      certificate_file,
    )
    .unwrap();
    reqwest::Certificate::from_pem(root_certificate.serialize_pem().unwrap().as_bytes()).unwrap()
  }
}

impl Drop for State {
  fn drop(&mut self) {
    if let Some(ord) = &mut self.ord {
      ord.kill().unwrap();
    }

    self.bitcoind.kill().unwrap();
  }
}
