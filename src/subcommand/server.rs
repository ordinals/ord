use super::*;

use async_rustls::rustls::Session;
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

// TODO:
// https://docs.rs/axum-server/latest/axum_server/fn.bind_rustls.html
// https://github.com/FlorianUekermann/rustls-acme
// https://docs.rs/rustls-acme/latest/rustls_acme/
// https://docs.rs/axum-server/latest/src/axum_server/tls_rustls/mod.rs.html#50-52
//
// Don't want to bother with nginx config or certbot on cronjob
//
// options:
// - use proxy (nginx, apache)
// - allow configuring `.well-known` path and use webroot plugin
// - integrate acme-rustls
//
// Server::bind

#[derive(Parser)]
pub(crate) struct Server {
  #[clap(long, default_value = "0.0.0.0")]
  address: String,
  #[clap(long, default_value = "80")]
  port: u16,
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

      let addr = (self.address, self.port)
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
