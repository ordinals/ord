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
  ord: Option<Child>,
}

static ONCE: Once = Once::new();

impl State {
  pub(crate) fn new() -> Self {
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
      bitcoind,
      client,
      wallet,
      ord: None,
      blockchain,
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
}

impl Drop for State {
  fn drop(&mut self) {
    if let Some(ord) = &mut self.ord {
      ord.kill().unwrap();
    }

    self.bitcoind.kill().unwrap();
  }
}
