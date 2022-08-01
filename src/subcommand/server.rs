use super::*;

use {
  self::tls_acceptor::TlsAcceptor,
  clap::ArgGroup,
  rustls_acme::{
    acme::{ACME_TLS_ALPN_NAME, LETS_ENCRYPT_PRODUCTION_DIRECTORY, LETS_ENCRYPT_STAGING_DIRECTORY},
    caches::DirCache,
    AcmeConfig,
  },
  tokio_stream::StreamExt,
};

mod tls_acceptor;

#[derive(Parser)]
#[clap(group = ArgGroup::new("port").multiple(false).required(true))]
pub(crate) struct Server {
  #[clap(
    long,
    default_value = "0.0.0.0",
    help = "Listen on <ADDRESS> for incoming requests."
  )]
  address: String,
  #[clap(
    long,
    help = "Request ACME TLS certificate for <ACME_DOMAIN>. This ord instance must be reachable at <ACME_DOMAIN>:443 to respond to Let's Encrypt ACME challenges."
  )]
  acme_domain: Vec<String>,
  #[clap(
    long,
    group = "port",
    help = "Listen on <HTTP_PORT> for incoming HTTP requests."
  )]
  http_port: Option<u16>,
  #[clap(
    long,
    group = "port",
    help = "Listen on <HTTPS_PORT> for incoming HTTPS requests.",
    requires_all = &["acme-cache", "acme-domain", "acme-contact"]
  )]
  https_port: Option<u16>,
  #[structopt(long, help = "Store ACME TLS certificates in <ACME_CACHE>.")]
  acme_cache: Option<PathBuf>,
  #[structopt(long, help = "Provide ACME contact <ACME_CONTACT>.")]
  acme_contact: Vec<String>,
}

impl Server {
  pub(crate) fn run(self, options: Options) -> Result {
    Runtime::new()?.block_on(async {
      let index = Arc::new(Index::open(&options)?);

      let clone = index.clone();
      thread::spawn(move || loop {
        if let Err(error) = clone.index_ranges() {
          log::error!("{error}");
        }
        thread::sleep(Duration::from_millis(100));
      });

      let app = Router::new()
        .route("/list/:outpoint", get(Self::list))
        .route("/status", get(Self::status))
        .layer(extract::Extension(index))
        .layer(
          CorsLayer::new()
            .allow_methods([http::Method::GET])
            .allow_origin(Any),
        );

      let (port, acceptor) = match (self.http_port, self.https_port) {
        (Some(http_port), None) => (http_port, None),
        (None, Some(https_port)) => {
          let config = AcmeConfig::new(self.acme_domain)
            .contact(self.acme_contact)
            .cache_option(Some(DirCache::new(self.acme_cache.unwrap())))
            .directory(if cfg!(test) {
              LETS_ENCRYPT_STAGING_DIRECTORY
            } else {
              LETS_ENCRYPT_PRODUCTION_DIRECTORY
            });

          let mut state = config.state();

          let acceptor = state.acceptor();

          tokio::spawn(async move {
            while let Some(result) = state.next().await {
              match result {
                Ok(ok) => log::info!("ACME event: {:?}", ok),
                Err(err) => log::error!("ACME error: {:?}", err),
              }
            }
          });

          (https_port, Some(acceptor))
        }
        (None, None) | (Some(_), Some(_)) => unreachable!(),
      };

      let addr = (self.address, port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow!("Failed to get socket addrs"))?;

      let handle = Handle::new();

      LISTENERS.lock().unwrap().push(handle.clone());

      let server = axum_server::Server::bind(addr).handle(handle);

      match acceptor {
        Some(acceptor) => {
          server
            .acceptor(TlsAcceptor(acceptor))
            .serve(app.into_make_service())
            .await?
        }
        None => server.serve(app.into_make_service()).await?,
      }

      Ok(())
    })
  }

  async fn list(
    extract::Path(outpoint): extract::Path<OutPoint>,
    index: extract::Extension<Arc<Index>>,
  ) -> impl IntoResponse {
    match index.list(outpoint) {
      Ok(Some(ranges)) => (StatusCode::OK, Json(Some(ranges))),
      Ok(None) => (StatusCode::NOT_FOUND, Json(None)),
      Err(error) => {
        eprintln!("Error serving request for outpoint {outpoint}: {error}");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
      }
    }
  }

  async fn status() -> StatusCode {
    StatusCode::OK
  }
}
