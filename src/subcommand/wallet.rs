use {
  super::*,
  crate::wallet::{
    inscribe::{
      batch::{Batch, Batchfile, Mode},
    },
    Wallet,
  },
  reqwest::Url,
};

pub mod balance;
pub mod cardinals;
pub mod create;
pub mod etch;
pub mod inscribe;
pub mod inscriptions;
pub mod outputs;
pub mod receive;
pub mod restore;
pub mod sats;
pub mod send;
pub mod transactions;

#[derive(Debug, Parser)]
pub(crate) struct WalletCommand {
  #[arg(long, default_value = "ord", help = "Use wallet named <WALLET>.")]
  pub(crate) name: String,
  #[arg(long, alias = "nosync", help = "Do not update index.")]
  pub(crate) no_sync: bool,
  #[command(subcommand)]
  pub(crate) subcommand: Subcommand,
}

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[command(about = "Get wallet balance")]
  Balance,
  #[command(about = "Create new wallet")]
  Create(create::Create),
  #[command(about = "Create rune")]
  Etch(etch::Etch),
  #[command(about = "Create inscription")]
  Inscribe(inscribe::Inscribe),
  #[command(about = "List wallet inscriptions")]
  Inscriptions,
  #[command(about = "Generate receive address")]
  Receive,
  #[command(about = "Restore wallet")]
  Restore(restore::Restore),
  #[command(about = "List wallet satoshis")]
  Sats(sats::Sats),
  #[command(about = "Send sat or inscription")]
  Send(send::Send),
  #[command(about = "See wallet transactions")]
  Transactions(transactions::Transactions),
  #[command(about = "List all unspent outputs in wallet")]
  Outputs,
  #[command(about = "List unspent cardinal outputs in wallet")]
  Cardinals,
}

impl WalletCommand {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let index = Arc::new(Index::open(&options)?);
    let handle = axum_server::Handle::new();
    LISTENERS.lock().unwrap().push(handle.clone());

    let ord_url: Url = {
      format!(
        "http://127.0.0.1:{}",
        TcpListener::bind("127.0.0.1:0")?.local_addr()?.port() // very hacky
      )
      .parse()
      .unwrap()
    };

    {
      let options = options.clone();
      let ord_url = ord_url.clone();
      std::thread::spawn(move || {
        crate::subcommand::server::Server {
          address: ord_url.host_str().map(|a| a.to_string()),
          acme_domain: vec![],
          csp_origin: None,
          http_port: ord_url.port(),
          https_port: None,
          acme_cache: None,
          acme_contact: vec![],
          http: true,
          https: false,
          redirect_http_to_https: false,
          enable_json_api: true,
          decompress: false,
          no_sync: self.no_sync,
        }
        .run(options, index, handle)
        .unwrap()
      });
    }

    let wallet = Wallet {
      no_sync: self.no_sync,
      options,
      ord_url,
      name: self.name.clone(),
    };

    let result = match self.subcommand {
      Subcommand::Balance => balance::run(wallet),
      Subcommand::Create(create) => create.run(wallet),
      Subcommand::Etch(etch) => etch.run(wallet),
      Subcommand::Inscribe(inscribe) => inscribe.run(wallet),
      Subcommand::Inscriptions => inscriptions::run(wallet),
      Subcommand::Receive => receive::run(wallet),
      Subcommand::Restore(restore) => restore.run(wallet),
      Subcommand::Sats(sats) => sats.run(wallet),
      Subcommand::Send(send) => send.run(wallet),
      Subcommand::Transactions(transactions) => transactions.run(wallet),
      Subcommand::Outputs => outputs::run(wallet),
      Subcommand::Cardinals => cardinals::run(wallet),
    };

    LISTENERS
      .lock()
      .unwrap()
      .iter()
      .for_each(|handle| handle.shutdown());

    result
  }
}
