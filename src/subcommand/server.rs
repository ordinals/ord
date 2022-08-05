use super::*;

use {
  self::{deserialize_ordinal_from_str::DeserializeOrdinalFromStr, tls_acceptor::TlsAcceptor},
  clap::ArgGroup,
  rustls_acme::{
    acme::{ACME_TLS_ALPN_NAME, LETS_ENCRYPT_PRODUCTION_DIRECTORY, LETS_ENCRYPT_STAGING_DIRECTORY},
    caches::DirCache,
    AcmeConfig,
  },
  serde::{de, Deserializer},
  tokio_stream::StreamExt,
};

mod deserialize_ordinal_from_str;
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
        .route("/", get(Self::root))
        .route("/api/list/:outpoint", get(Self::api_list))
        .route("/block/:hash", get(Self::block))
        .route("/ordinal/:ordinal", get(Self::ordinal))
        .route("/output/:output", get(Self::output))
        .route("/range/:start/:end", get(Self::range))
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

  async fn ordinal(
    extract::Path(DeserializeOrdinalFromStr(ordinal)): extract::Path<DeserializeOrdinalFromStr>,
  ) -> impl IntoResponse {
    (StatusCode::OK, Html(format!("{ordinal}")))
  }

  async fn output(
    index: extract::Extension<Arc<Index>>,
    extract::Path(outpoint): extract::Path<OutPoint>,
  ) -> impl IntoResponse {
    match index.list(outpoint) {
      Ok(Some(ranges)) => (
        StatusCode::OK,
        Html(format!(
          "<ul>{}</ul>",
          ranges
            .iter()
            .map(|(start, end)| format!(
              "<li><a href='/range/{start}/{end}'>[{start},{end})</a></li>"
            ))
            .collect::<String>()
        )),
      ),
      Ok(None) => (
        StatusCode::NOT_FOUND,
        Html("Output unknown, invalid, or spent.".to_string()),
      ),
      Err(err) => {
        eprintln!("Error serving request for output: {err}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Html(
            StatusCode::INTERNAL_SERVER_ERROR
              .canonical_reason()
              .unwrap_or_default()
              .to_string(),
          ),
        )
      }
    }
  }

  async fn range(
    extract::Path((DeserializeOrdinalFromStr(start), DeserializeOrdinalFromStr(end))): extract::Path<
      (DeserializeOrdinalFromStr, DeserializeOrdinalFromStr),
    >,
  ) -> impl IntoResponse {
    if start == end {
      return (StatusCode::BAD_REQUEST, Html("Empty Range".to_string()));
    }

    if start > end {
      return (
        StatusCode::BAD_REQUEST,
        Html("Range Start Greater Than Range End".to_string()),
      );
    }

    (
      StatusCode::OK,
      Html(format!("<a href='/ordinal/{start}'>first</a>")),
    )
  }

  async fn root(index: extract::Extension<Arc<Index>>) -> impl IntoResponse {
    match index.all() {
      Ok(blocks) => (
        StatusCode::OK,
        Html(format!(
          "<ul>\n{}</ul>",
          blocks
            .iter()
            .enumerate()
            .map(|(height, hash)| format!(
              "  <li>{height} - <a href='/block/{hash}'>{hash}</a></li>\n"
            ))
            .collect::<String>(),
        )),
      ),
      Err(error) => {
        eprintln!("Error serving request for root: {error}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Html(
            StatusCode::INTERNAL_SERVER_ERROR
              .canonical_reason()
              .unwrap_or_default()
              .to_string(),
          ),
        )
      }
    }
  }

  async fn block(
    extract::Path(hash): extract::Path<sha256d::Hash>,
    index: extract::Extension<Arc<Index>>,
  ) -> impl IntoResponse {
    match index.block_with_hash(hash) {
      Ok(Some(block)) => (
        StatusCode::OK,
        Html(format!(
          "<ul>\n{}</ul>",
          block
            .txdata
            .iter()
            .enumerate()
            .map(|(i, tx)| format!("  <li>{i} - {}</li>\n", tx.txid()))
            .collect::<String>()
        )),
      ),
      Ok(None) => (
        StatusCode::NOT_FOUND,
        Html(
          StatusCode::NOT_FOUND
            .canonical_reason()
            .unwrap_or_default()
            .to_string(),
        ),
      ),
      Err(error) => {
        eprintln!("Error serving request for block with hash {hash}: {error}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Html(
            StatusCode::INTERNAL_SERVER_ERROR
              .canonical_reason()
              .unwrap_or_default()
              .to_string(),
          ),
        )
      }
    }
  }

  async fn api_list(
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
