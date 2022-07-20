use super::*;

use async_rustls::rustls::Session;
use clap::ArgGroup;
use futures_util::future::poll_fn;
use hyper::server::accept::Accept;
use hyper::server::conn::AddrIncoming;
use hyper::server::conn::Http;
use rustls_acme::acme::ACME_TLS_ALPN_NAME;
use std::net::SocketAddr;
use std::pin::Pin;
use tokio_stream::StreamExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tower::MakeService;

// #[structopt(
//   long,
//   help = "Store TLS certificates fetched from Let's Encrypt via the ACME protocol in <acme-cache-directory>."
// )]
// pub(crate) acme_cache_directory: Option<PathBuf>,
// #[structopt(
//   long,
//   help = "Request TLS certificate for <acme-domain>. This agora instance must be reachable at <acme-domain>:443 to respond to Let's Encrypt ACME challenges."
// )]
// pub(crate) acme_domain: Vec<String>,
// #[structopt(
//   long,
//   default_value = "0.0.0.0",
//   help = "Listen on <address> for incoming requests."
// )]
// pub(crate) address: String,
// #[structopt(
//   long,
//   group = "port",
//   help = "Listen on <http-port> for incoming HTTP requests."
// )]
// pub(crate) http_port: Option<u16>,
// #[structopt(
//   long,
//   group = "port",
//   help = "Listen on <https-port> for incoming HTTPS requests.",
//   requires_all = &["acme-cache-directory", "acme-domain"]
// )]
// pub(crate) https_port: Option<u16>,
// #[structopt(
//   long,
//   help = "Redirect HTTP requests on <https-redirect-port> to HTTPS on <https-port>.",
//   requires = "https-port"
// )]
// pub(crate) https_redirect_port: Option<u16>,

// TODO:
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
    group = "port",
    help = "Listen on <HTTP_PORT> for incoming HTTP requests."
  )]
  http_port: Option<u16>,
  #[clap(
    long,
    group = "port",
    help = "Listen on <HTTPS_PORT> for incoming HTTPS requests."
  )]
  https_port: Option<u16>,
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

      match (self.http_port, self.https_port) {
        (Some(http_port), None) => {
          let app = Router::new()
            .route("/list/:outpoint", get(Self::list))
            .route("/status", get(Self::status))
            .layer(extract::Extension(index))
            .layer(
              CorsLayer::new()
                .allow_methods([http::Method::GET])
                .allow_origin(Any),
            );

          let addr = (self.address, http_port)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow!("Failed to get socket addrs"))?;

          let handle = Handle::new();

          LISTENERS.lock().unwrap().push(handle.clone());

          axum_server::Server::bind(addr)
            .handle(handle)
            .serve(app.into_make_service())
            .await?;
        }
        (None, Some(https_port)) => {
          let mut app = Router::new()
            .route("/list/:outpoint", get(Self::list))
            .route("/status", get(Self::status))
            .layer(extract::Extension(index))
            .layer(
              CorsLayer::new()
                .allow_methods([http::Method::GET])
                .allow_origin(Any),
            )
            .into_make_service_with_connect_info::<SocketAddr>();

          let addr = (self.address, https_port)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow!("Failed to get socket addrs"))?;

          let handle = Handle::new();

          LISTENERS.lock().unwrap().push(handle.clone());

          use rustls_acme::{caches::DirCache, AcmeConfig};

          let config = AcmeConfig::new(["api.ordinals.com"])
            .contact(["mailto:casey@rodarmor.com"])
            .cache_option(Some(DirCache::new("/Users/rodarmor/tmp/acme-cache")))
            .directory(if cfg!(test) {
              LETS_ENCRYPT_STAGING_DIRECTORY
            } else {
              LETS_ENCRYPT_PRODUCTION_DIRECTORY
            });

          let mut state = config.state();

          let acceptor = state.acceptor();

          tokio::spawn(async move {
            loop {
              match state.next().await.unwrap() {
                Ok(ok) => log::info!("ACME event: {:?}", ok),
                Err(err) => log::error!("ACME error: {:?}", err),
              }
            }
          });

          let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
          let mut addr_incoming = AddrIncoming::from_listener(listener).unwrap();

          loop {
            let stream = poll_fn(|cx| Pin::new(&mut addr_incoming).poll_accept(cx))
              .await
              .unwrap()
              .unwrap();
            let acceptor = acceptor.clone();

            let app = app.make_service(&stream).await.unwrap();

            tokio::spawn(async move {
              let tls = acceptor.accept(stream.compat()).await.unwrap().compat();
              match tls.get_ref().get_ref().1.get_alpn_protocol() {
                Some(ACME_TLS_ALPN_NAME) => log::info!("received TLS-ALPN-01 validation request"),
                _ => Http::new().serve_connection(tls, app).await.unwrap(),
              }
            });
          }
        }
        (None, None) | (Some(_), Some(_)) => unreachable!(),
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
