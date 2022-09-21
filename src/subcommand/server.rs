use super::*;

use {
  self::{
    deserialize_ordinal_from_str::DeserializeOrdinalFromStr,
    templates::{
      block::BlockHtml, clock::ClockSvg, home::HomeHtml, ordinal::OrdinalHtml, output::OutputHtml,
      range::RangeHtml, transaction::TransactionHtml, Content,
    },
  },
  axum::{
    body,
    http::header,
    response::{Redirect, Response},
  },
  rust_embed::RustEmbed,
  rustls_acme::{
    acme::{LETS_ENCRYPT_PRODUCTION_DIRECTORY, LETS_ENCRYPT_STAGING_DIRECTORY},
    axum::AxumAcceptor,
    caches::DirCache,
    AcmeConfig,
  },
  serde::{de, Deserializer},
  std::cmp::Ordering,
  tokio_stream::StreamExt,
};

mod deserialize_ordinal_from_str;
mod templates;

#[derive(Deserialize)]
struct Search {
  query: String,
}

#[derive(RustEmbed)]
#[folder = "static"]
struct StaticAssets;

struct StaticHtml {
  title: &'static str,
  html: &'static str,
}

impl Content for StaticHtml {
  fn title(&self) -> String {
    self.title.into()
  }
}

impl Display for StaticHtml {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.write_str(self.html)
  }
}

#[derive(Debug, Parser)]
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
    help = "Listen on <HTTP_PORT> for incoming HTTP requests. [default: 80]."
  )]
  http_port: Option<u16>,
  #[clap(
    long,
    group = "port",
    help = "Listen on <HTTPS_PORT> for incoming HTTPS requests. [default: 443]."
  )]
  https_port: Option<u16>,
  #[clap(long, help = "Store ACME TLS certificates in <ACME_CACHE>.")]
  acme_cache: Option<PathBuf>,
  #[clap(long, help = "Provide ACME contact <ACME_CONTACT>.")]
  acme_contact: Vec<String>,
  #[clap(long, help = "Serve HTTP traffic on <HTTP_PORT>.")]
  http: bool,
  #[clap(long, help = "Serve HTTPS traffic on <HTTPS_PORT>.")]
  https: bool,
}

impl Server {
  pub(crate) fn run(self, options: Options) -> Result {
    Runtime::new()?.block_on(async {
      let index = Arc::new(Index::open(&options)?);

      let clone = index.clone();
      thread::spawn(move || loop {
        if let Err(error) = clone.index() {
          log::error!("{error}");
        }
        thread::sleep(Duration::from_millis(100));
      });

      let router = Router::new()
        .route("/", get(Self::home))
        .route("/block/:hash", get(Self::block))
        .route("/bounties", get(Self::bounties))
        .route("/clock", get(Self::clock))
        .route("/clock.svg", get(Self::clock))
        .route("/faq", get(Self::faq))
        .route("/favicon.ico", get(Self::favicon))
        .route("/height", get(Self::height))
        .route("/ordinal/:ordinal", get(Self::ordinal))
        .route("/output/:output", get(Self::output))
        .route("/range/:start/:end", get(Self::range))
        .route("/search", get(Self::search_by_query))
        .route("/search/:query", get(Self::search_by_path))
        .route("/static/*path", get(Self::static_asset))
        .route("/status", get(Self::status))
        .route("/tx/:txid", get(Self::transaction))
        .layer(extract::Extension(index))
        .layer(
          CorsLayer::new()
            .allow_methods([http::Method::GET])
            .allow_origin(Any),
        );

      let handle = Handle::new();

      LISTENERS.lock().unwrap().push(handle.clone());

      let (http_result, https_result) = tokio::join!(
        self.spawn(&router, &handle, None)?,
        self.spawn(&router, &handle, self.acceptor(&options)?)?
      );

      http_result.and(https_result)?.transpose()?;

      Ok(())
    })
  }

  fn spawn(
    &self,
    router: &Router,
    handle: &Handle,
    https_acceptor: Option<AxumAcceptor>,
  ) -> Result<task::JoinHandle<Option<io::Result<()>>>> {
    let addr = if https_acceptor.is_some() {
      self.https_port()
    } else {
      self.http_port()
    }
    .map(|port| {
      (self.address.as_str(), port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow!("Failed to get socket addrs"))
        .map(|addr| (addr, router.clone(), handle.clone()))
    })
    .transpose()?;

    Ok(tokio::spawn(async move {
      if let Some((addr, router, handle)) = addr {
        Some(if let Some(acceptor) = https_acceptor {
          axum_server::Server::bind(addr)
            .handle(handle)
            .acceptor(acceptor)
            .serve(router.into_make_service())
            .await
        } else {
          axum_server::Server::bind(addr)
            .handle(handle)
            .serve(router.into_make_service())
            .await
        })
      } else {
        None
      }
    }))
  }

  fn acme_cache(acme_cache: Option<&PathBuf>, options: &Options) -> Result<PathBuf> {
    if let Some(acme_cache) = acme_cache {
      Ok(acme_cache.clone())
    } else {
      Ok(options.data_dir()?.join("acme-cache"))
    }
  }

  fn acme_domains(acme_domain: &Vec<String>) -> Result<Vec<String>> {
    if !acme_domain.is_empty() {
      Ok(acme_domain.clone())
    } else {
      Ok(vec![sys_info::hostname()?])
    }
  }

  fn http_port(&self) -> Option<u16> {
    if self.http || self.http_port.is_some() || (self.https_port.is_none() && !self.https) {
      Some(self.http_port.unwrap_or(80))
    } else {
      None
    }
  }

  fn https_port(&self) -> Option<u16> {
    if self.https || self.https_port.is_some() {
      Some(self.https_port.unwrap_or(443))
    } else {
      None
    }
  }

  fn acceptor(&self, options: &Options) -> Result<Option<AxumAcceptor>> {
    if self.https_port().is_some() {
      let config = AcmeConfig::new(Self::acme_domains(&self.acme_domain)?)
        .contact(&self.acme_contact)
        .cache_option(Some(DirCache::new(Self::acme_cache(
          self.acme_cache.as_ref(),
          options,
        )?)))
        .directory(if cfg!(test) {
          LETS_ENCRYPT_STAGING_DIRECTORY
        } else {
          LETS_ENCRYPT_PRODUCTION_DIRECTORY
        });

      let mut state = config.state();

      let acceptor = state.axum_acceptor(Arc::new(
        rustls::ServerConfig::builder()
          .with_safe_defaults()
          .with_no_client_auth()
          .with_cert_resolver(state.resolver()),
      ));

      tokio::spawn(async move {
        while let Some(result) = state.next().await {
          match result {
            Ok(ok) => log::info!("ACME event: {:?}", ok),
            Err(err) => log::error!("ACME error: {:?}", err),
          }
        }
      });

      Ok(Some(acceptor))
    } else {
      Ok(None)
    }
  }

  async fn clock(index: extract::Extension<Arc<Index>>) -> impl IntoResponse {
    match index.height() {
      Ok(height) => ClockSvg::new(height).into_response(),
      Err(err) => {
        eprintln!("Failed to retrieve height from index: {err}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Html(
            StatusCode::INTERNAL_SERVER_ERROR
              .canonical_reason()
              .unwrap_or_default()
              .to_string(),
          ),
        )
          .into_response()
      }
    }
  }

  async fn ordinal(
    index: extract::Extension<Arc<Index>>,
    extract::Path(DeserializeOrdinalFromStr(ordinal)): extract::Path<DeserializeOrdinalFromStr>,
  ) -> impl IntoResponse {
    match index.blocktime(ordinal.height()) {
      Ok(blocktime) => OrdinalHtml { ordinal, blocktime }.page().into_response(),
      Err(err) => {
        eprintln!("Failed to retrieve blocktime from index: {err}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Html(
            StatusCode::INTERNAL_SERVER_ERROR
              .canonical_reason()
              .unwrap_or_default()
              .to_string(),
          ),
        )
          .into_response()
      }
    }
  }

  async fn output(
    index: extract::Extension<Arc<Index>>,
    extract::Path(outpoint): extract::Path<OutPoint>,
  ) -> impl IntoResponse {
    match index.list(outpoint) {
      Ok(Some(list)) => OutputHtml { outpoint, list }.page().into_response(),
      Ok(None) => (StatusCode::NOT_FOUND, Html("Output unknown.".to_string())).into_response(),
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
          .into_response()
      }
    }
  }

  async fn range(
    extract::Path((DeserializeOrdinalFromStr(start), DeserializeOrdinalFromStr(end))): extract::Path<
      (DeserializeOrdinalFromStr, DeserializeOrdinalFromStr),
    >,
  ) -> impl IntoResponse {
    match start.cmp(&end) {
      Ordering::Equal => (StatusCode::BAD_REQUEST, Html("Empty Range".to_string())).into_response(),
      Ordering::Greater => (
        StatusCode::BAD_REQUEST,
        Html("Range Start Greater Than Range End".to_string()),
      )
        .into_response(),
      Ordering::Less => RangeHtml { start, end }.page().into_response(),
    }
  }

  async fn home(index: extract::Extension<Arc<Index>>) -> impl IntoResponse {
    match index.blocks(100) {
      Ok(blocks) => HomeHtml::new(blocks).page().into_response(),
      Err(err) => {
        eprintln!("Error getting blocks: {err}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Html(
            StatusCode::INTERNAL_SERVER_ERROR
              .canonical_reason()
              .unwrap_or_default()
              .to_string(),
          ),
        )
          .into_response()
      }
    }
  }

  async fn block(
    extract::Path(hash): extract::Path<sha256d::Hash>,
    index: extract::Extension<Arc<Index>>,
  ) -> impl IntoResponse {
    match index.block_with_hash(hash) {
      Ok(Some(block)) => BlockHtml::new(block).page().into_response(),
      Ok(None) => (
        StatusCode::NOT_FOUND,
        Html(
          StatusCode::NOT_FOUND
            .canonical_reason()
            .unwrap_or_default()
            .to_string(),
        ),
      )
        .into_response(),
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
          .into_response()
      }
    }
  }

  async fn transaction(
    index: extract::Extension<Arc<Index>>,
    extract::Path(txid): extract::Path<Txid>,
  ) -> impl IntoResponse {
    match index.transaction(txid) {
      Ok(Some(transaction)) => TransactionHtml::new(transaction).page().into_response(),
      Ok(None) => (
        StatusCode::NOT_FOUND,
        Html(
          StatusCode::NOT_FOUND
            .canonical_reason()
            .unwrap_or_default()
            .to_string(),
        ),
      )
        .into_response(),
      Err(error) => {
        eprintln!("Error serving request for transaction with txid {txid}: {error}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Html(
            StatusCode::INTERNAL_SERVER_ERROR
              .canonical_reason()
              .unwrap_or_default()
              .to_string(),
          ),
        )
          .into_response()
      }
    }
  }

  async fn status() -> impl IntoResponse {
    (
      StatusCode::OK,
      StatusCode::OK
        .canonical_reason()
        .unwrap_or_default()
        .to_string(),
    )
  }

  async fn search_by_query(search: extract::Query<Search>) -> Redirect {
    Redirect::to(&format!("/ordinal/{}", search.query))
  }

  async fn search_by_path(search: extract::Path<Search>) -> Redirect {
    Redirect::to(&format!("/ordinal/{}", search.query))
  }

  async fn favicon() -> impl IntoResponse {
    Self::static_asset(extract::Path("/favicon.png".to_string())).await
  }

  async fn static_asset(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
    match StaticAssets::get(if let Some(stripped) = path.strip_prefix('/') {
      stripped
    } else {
      &path
    }) {
      Some(content) => {
        let body = body::boxed(body::Full::from(content.data));
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        Response::builder()
          .header(header::CONTENT_TYPE, mime.as_ref())
          .body(body)
          .unwrap()
      }
      None => (
        StatusCode::NOT_FOUND,
        Html(
          StatusCode::NOT_FOUND
            .canonical_reason()
            .unwrap_or_default()
            .to_string(),
        ),
      )
        .into_response(),
    }
  }

  async fn height(index: extract::Extension<Arc<Index>>) -> impl IntoResponse {
    match index.height() {
      Ok(height) => (StatusCode::OK, Html(format!("{}", height))),
      Err(err) => {
        eprintln!("Failed to retrieve height from index: {err}");
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

  async fn faq() -> impl IntoResponse {
    Redirect::to("https://docs.ordinals.com/faq/")
  }

  async fn bounties() -> impl IntoResponse {
    Redirect::to("https://docs.ordinals.com/bounties/")
  }
}

#[cfg(test)]
mod tests {
  use {super::*, std::net::TcpListener, tempfile::TempDir};

  struct TestServer {
    #[allow(unused)]
    bitcoin_rpc_server_handle: BitcoinRpcServerHandle,
    #[allow(unused)]
    tempdir: TempDir,
    port: u16,
  }

  impl TestServer {
    fn new() -> Self {
      let bitcoin_rpc_server_handle = BitcoinRpcServer::spawn();

      let tempdir = TempDir::new().unwrap();

      let cookiefile = tempdir.path().join("cookie");

      fs::write(&cookiefile, "username:password").unwrap();

      let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

      let (options, server) = parse_server_args(&format!(
        "ord --chain regtest --rpc-url http://127.0.0.1:{} --cookie-file {} --data-dir {} server --http-port {} --address 127.0.0.1",
        bitcoin_rpc_server_handle.port,
        cookiefile.to_str().unwrap(),
        tempdir.path().to_str().unwrap(),
        port,
      ));

      thread::spawn(|| server.run(options).unwrap());

      for i in 0.. {
        match reqwest::blocking::get(&format!("http://127.0.0.1:{port}/status")) {
          Ok(_) => break,
          Err(err) => {
            if i == 400 {
              panic!("Server failed to start: {err}");
            }
          }
        }

        thread::sleep(Duration::from_millis(25));
      }

      Self {
        bitcoin_rpc_server_handle,
        port,
        tempdir,
      }
    }

    fn join_url(&self, url: &str) -> String {
      format!("http://127.0.0.1:{}/{url}", self.port)
    }
  }

  fn parse_server_args(args: &str) -> (Options, Server) {
    match Arguments::try_parse_from(args.split_whitespace()) {
      Ok(arguments) => match arguments.subcommand {
        Subcommand::Server(server) => (arguments.options, server),
        subcommand => panic!("Unexpected subcommand: {subcommand:?}"),
      },
      Err(err) => panic!("Error parsing arguments: {err}"),
    }
  }

  #[test]
  fn http_and_https_port_dont_conflict() {
    parse_server_args(
      "ord server --http-port 0 --https-port 0 --acme-cache foo --acme-contact bar --acme-domain baz",
    );
  }

  #[test]
  fn http_port_defaults_to_80() {
    assert_eq!(parse_server_args("ord server").1.http_port(), Some(80));
  }

  #[test]
  fn https_port_defaults_to_none() {
    assert_eq!(parse_server_args("ord server").1.https_port(), None);
  }

  #[test]
  fn https_sets_https_port_to_443() {
    assert_eq!(
      parse_server_args("ord server --https --acme-cache foo --acme-contact bar --acme-domain baz")
        .1
        .https_port(),
      Some(443)
    );
  }

  #[test]
  fn https_disables_http() {
    assert_eq!(
      parse_server_args("ord server --https --acme-cache foo --acme-contact bar --acme-domain baz")
        .1
        .http_port(),
      None
    );
  }

  #[test]
  fn https_port_disables_http() {
    assert_eq!(
      parse_server_args(
        "ord server --https-port 433 --acme-cache foo --acme-contact bar --acme-domain baz"
      )
      .1
      .http_port(),
      None
    );
  }

  #[test]
  fn https_port_sets_https_port() {
    assert_eq!(
      parse_server_args(
        "ord server --https-port 1000 --acme-cache foo --acme-contact bar --acme-domain baz"
      )
      .1
      .https_port(),
      Some(1000)
    );
  }

  #[test]
  fn http_with_https_leaves_http_enabled() {
    assert_eq!(
      parse_server_args(
        "ord server --https --http --acme-cache foo --acme-contact bar --acme-domain baz"
      )
      .1
      .http_port(),
      Some(80)
    );
  }

  #[test]
  fn http_with_https_leaves_https_enabled() {
    assert_eq!(
      parse_server_args(
        "ord server --https --http --acme-cache foo --acme-contact bar --acme-domain baz"
      )
      .1
      .https_port(),
      Some(443)
    );
  }

  #[test]
  fn acme_contact_accepts_multiple_values() {
    assert!(Arguments::try_parse_from(&[
      "ord",
      "server",
      "--address",
      "127.0.0.1",
      "--http-port",
      "0",
      "--acme-contact",
      "foo",
      "--acme-contact",
      "bar"
    ])
    .is_ok());
  }

  #[test]
  fn acme_domain_accepts_multiple_values() {
    assert!(Arguments::try_parse_from(&[
      "ord",
      "server",
      "--address",
      "127.0.0.1",
      "--http-port",
      "0",
      "--acme-domain",
      "foo",
      "--acme-domain",
      "bar"
    ])
    .is_ok());
  }

  #[test]
  fn acme_cache_defaults_to_data_dir() {
    let arguments = Arguments::try_parse_from(&["ord", "--data-dir", "foo", "server"]).unwrap();
    let acme_cache = Server::acme_cache(None, &arguments.options)
      .unwrap()
      .display()
      .to_string();
    assert!(acme_cache.contains("foo/acme-cache"), "{acme_cache}")
  }

  #[test]
  fn acme_cache_flag_is_respected() {
    let arguments =
      Arguments::try_parse_from(&["ord", "--data-dir", "foo", "server", "--acme-cache", "bar"])
        .unwrap();
    let acme_cache = Server::acme_cache(Some(&"bar".into()), &arguments.options)
      .unwrap()
      .display()
      .to_string();
    assert_eq!(acme_cache, "bar")
  }

  #[test]
  fn acme_domain_defaults_to_hostname() {
    assert_eq!(
      Server::acme_domains(&Vec::new()).unwrap(),
      &[sys_info::hostname().unwrap()]
    );
  }

  #[test]
  fn acme_domain_flag_is_respected() {
    assert_eq!(
      Server::acme_domains(&vec!["example.com".into()]).unwrap(),
      &["example.com"]
    );
  }

  #[test]
  fn bounties_redirects_to_docs_site() {
    let test_server = TestServer::new();

    let response = reqwest::blocking::Client::builder()
      .redirect(reqwest::redirect::Policy::none())
      .build()
      .unwrap()
      .get(test_server.join_url("bounties"))
      .send()
      .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
      response.headers().get(header::LOCATION).unwrap(),
      "https://docs.ordinals.com/bounties/"
    );
  }

  #[test]
  fn faq_redirects_to_docs_site() {
    let test_server = TestServer::new();

    let response = reqwest::blocking::Client::builder()
      .redirect(reqwest::redirect::Policy::none())
      .build()
      .unwrap()
      .get(test_server.join_url("faq"))
      .send()
      .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
      response.headers().get(header::LOCATION).unwrap(),
      "https://docs.ordinals.com/faq/"
    );
  }

  #[test]
  fn search_by_query_returns_ordinal() {
    let test_server = TestServer::new();

    let response = reqwest::blocking::Client::builder()
      .redirect(reqwest::redirect::Policy::none())
      .build()
      .unwrap()
      .get(test_server.join_url("search?query=0"))
      .send()
      .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
      response.headers().get(header::LOCATION).unwrap(),
      "/ordinal/0"
    );
  }

  #[test]
  fn search_by_path_returns_ordinal() {
    let test_server = TestServer::new();

    let response = reqwest::blocking::Client::builder()
      .redirect(reqwest::redirect::Policy::none())
      .build()
      .unwrap()
      .get(test_server.join_url("search/0"))
      .send()
      .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
      response.headers().get(header::LOCATION).unwrap(),
      "/ordinal/0"
    );
  }
}
