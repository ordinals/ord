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
  lazy_static::lazy_static,
  rust_embed::RustEmbed,
  rustls_acme::{
    acme::{LETS_ENCRYPT_PRODUCTION_DIRECTORY, LETS_ENCRYPT_STAGING_DIRECTORY},
    axum::AxumAcceptor,
    caches::DirCache,
    AcmeConfig,
  },
  serde::{de, Deserializer},
  std::cmp::Ordering,
  std::str,
  tokio_stream::StreamExt,
};

mod deserialize_ordinal_from_str;
mod templates;

fn html_status(status_code: StatusCode) -> (StatusCode, Html<&'static str>) {
  (
    status_code,
    Html(status_code.canonical_reason().unwrap_or_default()),
  )
}

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
  pub(crate) fn run(self, options: Options, index: Arc<Index>, handle: Handle) -> Result {
    Runtime::new()?.block_on(async {
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
        .layer(extract::Extension(options.chain.network()))
        .layer(
          CorsLayer::new()
            .allow_methods([http::Method::GET])
            .allow_origin(Any),
        );

      match (self.http_port(), self.https_port()) {
        (Some(http_port), None) => self.spawn(router, handle, http_port, None)?.await??,
        (None, Some(https_port)) => {
          self
            .spawn(router, handle, https_port, Some(self.acceptor(&options)?))?
            .await??
        }
        (Some(http_port), Some(https_port)) => {
          let (http_result, https_result) = tokio::join!(
            self.spawn(router.clone(), handle.clone(), http_port, None)?,
            self.spawn(router, handle, https_port, Some(self.acceptor(&options)?))?
          );
          http_result.and(https_result)??;
        }
        (None, None) => unreachable!(),
      }

      Ok(())
    })
  }

  fn spawn(
    &self,
    router: Router,
    handle: Handle,
    port: u16,
    https_acceptor: Option<AxumAcceptor>,
  ) -> Result<task::JoinHandle<io::Result<()>>> {
    let addr = (self.address.as_str(), port)
      .to_socket_addrs()?
      .next()
      .ok_or_else(|| anyhow!("Failed to get socket addrs"))?;

    Ok(tokio::spawn(async move {
      if let Some(acceptor) = https_acceptor {
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

  fn acceptor(&self, options: &Options) -> Result<AxumAcceptor> {
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

    Ok(acceptor)
  }

  async fn clock(index: extract::Extension<Arc<Index>>) -> impl IntoResponse {
    match index.height() {
      Ok(height) => ClockSvg::new(height).into_response(),
      Err(err) => {
        eprintln!("Failed to retrieve height from index: {err}");
        html_status(StatusCode::INTERNAL_SERVER_ERROR).into_response()
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
        html_status(StatusCode::INTERNAL_SERVER_ERROR).into_response()
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
        html_status(StatusCode::INTERNAL_SERVER_ERROR).into_response()
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
        html_status(StatusCode::INTERNAL_SERVER_ERROR).into_response()
      }
    }
  }

  async fn block(
    extract::Path(hash): extract::Path<BlockHash>,
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
        html_status(StatusCode::INTERNAL_SERVER_ERROR).into_response()
      }
    }
  }

  async fn transaction(
    index: extract::Extension<Arc<Index>>,
    network: extract::Extension<Network>,
    extract::Path(txid): extract::Path<Txid>,
  ) -> impl IntoResponse {
    match index.transaction(txid) {
      Ok(Some(transaction)) => TransactionHtml::new(transaction, network.0)
        .page()
        .into_response(),
      Ok(None) => html_status(StatusCode::NOT_FOUND).into_response(),
      Err(error) => {
        eprintln!("Error serving request for transaction with txid {txid}: {error}");
        html_status(StatusCode::INTERNAL_SERVER_ERROR).into_response()
      }
    }
  }

  async fn status(index: extract::Extension<Arc<Index>>) -> impl IntoResponse {
    if index.is_reorged() {
      (
        StatusCode::OK,
        "Reorg detected, please rebuild the database.",
      )
    } else {
      (
        StatusCode::OK,
        StatusCode::OK.canonical_reason().unwrap_or_default(),
      )
    }
  }

  async fn search_by_query(
    index: extract::Extension<Arc<Index>>,
    search: extract::Query<Search>,
  ) -> impl IntoResponse {
    Self::search(&index.0, &search.0.query).await
  }

  async fn search_by_path(
    index: extract::Extension<Arc<Index>>,
    search: extract::Path<Search>,
  ) -> impl IntoResponse {
    Self::search(&index.0, &search.0.query).await
  }

  async fn search(index: &Index, query: &str) -> Response {
    match Self::search_inner(index, query) {
      Ok(redirect) => redirect.into_response(),
      Err(err) => (StatusCode::BAD_REQUEST, Html(err.to_string())).into_response(),
    }
  }

  fn search_inner(index: &Index, query: &str) -> Result<Redirect> {
    lazy_static! {
      static ref HASH: Regex = Regex::new(r"^[[:xdigit:]]{64}$").unwrap();
      static ref OUTPOINT: Regex = Regex::new(r"^[[:xdigit:]]{64}:\d+$").unwrap();
    }

    let query = query.trim();

    if HASH.is_match(query) {
      if index.block_header(query.parse()?)?.is_some() {
        Ok(Redirect::to(&format!("/block/{query}")))
      } else {
        Ok(Redirect::to(&format!("/tx/{query}")))
      }
    } else if OUTPOINT.is_match(query) {
      Ok(Redirect::to(&format!("/output/{query}")))
    } else {
      Ok(Redirect::to(&format!("/ordinal/{query}")))
    }
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
    Redirect::to("https://docs.ordinals.com/bounty/")
  }
}

#[cfg(test)]
mod tests {
  use {super::*, reqwest::Url, std::net::TcpListener, tempfile::TempDir};

  struct TestServer {
    bitcoin_rpc_server: test_bitcoincore_rpc::Handle,
    index: Arc<Index>,
    ord_server_handle: Handle,
    url: Url,
    #[allow(unused)]
    tempdir: TempDir,
  }

  impl TestServer {
    fn new() -> Self {
      let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

      let tempdir = TempDir::new().unwrap();

      let cookiefile = tempdir.path().join("cookie");

      fs::write(&cookiefile, "username:password").unwrap();

      let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

      let url = Url::parse(&format!("http://127.0.0.1:{port}")).unwrap();

      let (options, server) = parse_server_args(&format!(
        "ord --chain regtest --rpc-url {} --cookie-file {} --data-dir {} server --http-port {} --address 127.0.0.1",
        bitcoin_rpc_server.url(),
        cookiefile.to_str().unwrap(),
        tempdir.path().to_str().unwrap(),
        port,
      ));

      let index = Arc::new(Index::open(&options).unwrap());
      let ord_server_handle = Handle::new();

      {
        let index = index.clone();
        let ord_server_handle = ord_server_handle.clone();
        thread::spawn(|| server.run(options, index, ord_server_handle).unwrap());
      }

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
        bitcoin_rpc_server,
        index,
        ord_server_handle,
        tempdir,
        url,
      }
    }

    fn get(&self, url: &str) -> reqwest::blocking::Response {
      if let Err(error) = self.index.index() {
        log::error!("{error}");
      }
      reqwest::blocking::get(self.join_url(url)).unwrap()
    }

    fn join_url(&self, url: &str) -> Url {
      self.url.join(url).unwrap()
    }

    fn assert_response(&self, path: &str, status: StatusCode, expected_response: &str) {
      let response = self.get(path);
      assert_eq!(response.status(), status);
      assert_eq!(response.text().unwrap(), expected_response);
    }

    fn assert_response_regex(&self, path: &str, status: StatusCode, regex: &str) {
      let response = self.get(path);
      assert_eq!(response.status(), status);
      assert_regex_match!(response.text().unwrap(), regex);
    }

    fn assert_redirect(&self, path: &str, location: &str) {
      let response = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
        .get(self.join_url(path))
        .send()
        .unwrap();

      assert_eq!(response.status(), StatusCode::SEE_OTHER);
      assert_eq!(response.headers().get(header::LOCATION).unwrap(), location);
    }
  }

  impl Drop for TestServer {
    fn drop(&mut self) {
      self.ord_server_handle.shutdown();
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
    TestServer::new().assert_redirect("/bounties", "https://docs.ordinals.com/bounty/");
  }

  #[test]
  fn faq_redirects_to_docs_site() {
    TestServer::new().assert_redirect("/faq", "https://docs.ordinals.com/faq/");
  }

  #[test]
  fn search_by_query_returns_ordinal() {
    TestServer::new().assert_redirect("/search?query=0", "/ordinal/0");
  }

  #[test]
  fn search_is_whitespace_insensitive() {
    TestServer::new().assert_redirect("/search/ 0 ", "/ordinal/0");
  }

  #[test]
  fn search_by_path_returns_ordinal() {
    TestServer::new().assert_redirect("/search/0", "/ordinal/0");
  }

  #[test]
  fn search_for_blockhash_returns_block() {
    TestServer::new().assert_redirect(
      "/search/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
      "/block/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
    );
  }

  #[test]
  fn search_for_txid_returns_transaction() {
    TestServer::new().assert_redirect(
      "/search/0000000000000000000000000000000000000000000000000000000000000000",
      "/tx/0000000000000000000000000000000000000000000000000000000000000000",
    );
  }

  #[test]
  fn search_for_outpoint_returns_output() {
    TestServer::new().assert_redirect(
      "/search/0000000000000000000000000000000000000000000000000000000000000000:0",
      "/output/0000000000000000000000000000000000000000000000000000000000000000:0",
    );
  }

  #[test]
  fn status() {
    TestServer::new().assert_response("/status", StatusCode::OK, "OK");
  }

  #[test]
  fn height_endpoint() {
    TestServer::new().assert_response("/height", StatusCode::OK, "0");
  }

  #[test]
  fn height_updates() {
    let test_server = TestServer::new();

    let response = test_server.get("/height");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "0");

    test_server.bitcoin_rpc_server.mine_blocks(1);

    let response = test_server.get("/height");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "1");
  }

  #[test]
  fn range_end_before_range_start_returns_400() {
    TestServer::new().assert_response(
      "/range/1/0/",
      StatusCode::BAD_REQUEST,
      "Range Start Greater Than Range End",
    );
  }

  #[test]
  fn invalid_range_start_returns_400() {
    TestServer::new().assert_response(
      "/range/=/0",
      StatusCode::BAD_REQUEST,
      "Invalid URL: invalid digit found in string",
    );
  }

  #[test]
  fn invalid_range_end_returns_400() {
    TestServer::new().assert_response(
      "/range/0/=",
      StatusCode::BAD_REQUEST,
      "Invalid URL: invalid digit found in string",
    );
  }

  #[test]
  fn empty_range_returns_400() {
    TestServer::new().assert_response("/range/0/0", StatusCode::BAD_REQUEST, "Empty Range");
  }

  #[test]
  fn range() {
    TestServer::new().assert_response_regex(
      "/range/0/1",
      StatusCode::OK,
      r".*<title>Ordinal range 0–1</title>.*<h1>Ordinal range 0–1</h1>
<dl>
  <dt>size</dt><dd>1</dd>
  <dt>first</dt><dd><a href=/ordinal/0 class=mythic>0</a></dd>
</dl>.*",
    );
  }
  #[test]
  fn ordinal_number() {
    TestServer::new().assert_response_regex("/ordinal/0", StatusCode::OK, ".*<h1>Ordinal 0</h1>.*");
  }

  #[test]
  fn ordinal_decimal() {
    TestServer::new().assert_response_regex(
      "/ordinal/0.0",
      StatusCode::OK,
      ".*<h1>Ordinal 0</h1>.*",
    );
  }

  #[test]
  fn ordinal_degree() {
    TestServer::new().assert_response_regex(
      "/ordinal/0°0′0″0‴",
      StatusCode::OK,
      ".*<h1>Ordinal 0</h1>.*",
    );
  }

  #[test]
  fn ordinal_name() {
    TestServer::new().assert_response_regex(
      "/ordinal/nvtdijuwxlp",
      StatusCode::OK,
      ".*<h1>Ordinal 0</h1>.*",
    );
  }

  #[test]
  fn ordinal() {
    TestServer::new().assert_response_regex(
      "/ordinal/0",
      StatusCode::OK,
      ".*<title>0°0′0″0‴</title>.*<h1>Ordinal 0</h1>.*",
    );
  }

  #[test]
  fn ordinal_out_of_range() {
    TestServer::new().assert_response(
      "/ordinal/2099999997690000",
      StatusCode::BAD_REQUEST,
      "Invalid URL: Invalid ordinal",
    );
  }

  #[test]
  fn invalid_outpoint_hash_returns_400() {
    TestServer::new().assert_response(
      "/output/foo:0",
      StatusCode::BAD_REQUEST,
      "Invalid URL: error parsing TXID",
    );
  }
  #[test]
  fn output() {
    let test_server = TestServer::new();

    test_server.assert_response_regex(
    "/output/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0",
    StatusCode::OK,
    ".*<title>Output 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0</title>.*<h1>Output 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0</h1>
<dl>
  <dt>size</dt><dd>5000000000</dd>
</dl>
<h2>Ordinal Ranges</h2>
<ul class=monospace>
  <li><a href=/range/0/5000000000 class=mythic>0–5000000000</a></li>
</ul>.*",
  );
  }

  #[test]
  fn unknown_output_returns_404() {
    TestServer::new().assert_response(
      "/output/0000000000000000000000000000000000000000000000000000000000000000:0",
      StatusCode::NOT_FOUND,
      "Output unknown.",
    );
  }

  #[test]
  fn invalid_output_returns_400() {
    TestServer::new().assert_response(
      "/output/foo:0",
      StatusCode::BAD_REQUEST,
      "Invalid URL: error parsing TXID",
    );
  }

  #[test]
  fn home() {
    let test_server = TestServer::new();

    test_server.bitcoin_rpc_server.mine_blocks(1);

    test_server.assert_response_regex(
    "/",
    StatusCode::OK,
    ".*<title>Ordinals</title>.*<h1>Ordinals</h1>
<nav>.*</nav>
.*
<h2>Latest Blocks</h2>
<dl>
  <dt>cycle</dt><dd>0</dd>
  <dt>epoch</dt><dd>0</dd>
  <dt>period</dt><dd>0</dd>
</dl>
<ol start=1 reversed class='blocks monospace'>
  <li><a href=/block/[[:xdigit:]]{64} class=uncommon>[[:xdigit:]]{64}</a></li>
  <li><a href=/block/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f class=mythic>000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f</a></li>
</ol>.*",
  );
  }

  #[test]
  fn home_block_limit() {
    let test_server = TestServer::new();

    test_server.bitcoin_rpc_server.mine_blocks(101);

    test_server.assert_response_regex(
    "/",
    StatusCode::OK,
    ".*<ol start=101 reversed class='blocks monospace'>\n(  <li><a href=/block/[[:xdigit:]]{64} class=uncommon>[[:xdigit:]]{64}</a></li>\n){100}</ol>.*"
  );
  }

  #[test]
  fn block_not_found() {
    TestServer::new().assert_response(
      "/block/467a86f0642b1d284376d13a98ef58310caa49502b0f9a560ee222e0a122fe16",
      StatusCode::NOT_FOUND,
      "Not Found",
    );
  }

  #[test]
  fn unmined_ordinal() {
    TestServer::new().assert_response_regex(
      "/ordinal/0",
      StatusCode::OK,
      ".*<dt>time</dt><dd>2009-01-03 18:15:05</dd>.*",
    );
  }

  #[test]
  fn mined_ordinal() {
    TestServer::new().assert_response_regex(
      "/ordinal/5000000000",
      StatusCode::OK,
      ".*<dt>time</dt><dd>.* \\(expected\\)</dd>.*",
    );
  }

  #[test]
  fn static_asset() {
    TestServer::new().assert_response_regex(
      "/static/index.css",
      StatusCode::OK,
      r".*\.rare \{
  background-color: cornflowerblue;
}.*",
    );
  }

  #[test]
  fn favicon() {
    TestServer::new().assert_response_regex("/favicon.ico", StatusCode::OK, r".*");
  }

  #[test]
  fn clock_updates() {
    let test_server = TestServer::new();
    test_server.assert_response_regex("/clock", StatusCode::OK, ".*<text.*>0</text>.*");
    test_server.bitcoin_rpc_server.mine_blocks(1);
    test_server.assert_response_regex("/clock", StatusCode::OK, ".*<text.*>1</text>.*");
  }

  #[test]
  fn block() {
    let test_server = TestServer::new();

    test_server.bitcoin_rpc_server.mine_blocks(1);
    let transaction = TransactionTemplate {
      input_slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 0,
    };
    test_server.bitcoin_rpc_server.broadcast_tx(transaction);
    let block_hash = test_server.bitcoin_rpc_server.mine_blocks(1)[0].block_hash();

    test_server.assert_response_regex(
      &format!("/block/{block_hash}"),
      StatusCode::OK,
      ".*<h1>Block [[:xdigit:]]{64}</h1>
<h2>Transactions</h2>
<ul class=monospace>
  <li><a href=/tx/[[:xdigit:]]{64}>[[:xdigit:]]{64}</a></li>
  <li><a href=/tx/[[:xdigit:]]{64}>[[:xdigit:]]{64}</a></li>
</ul>.*",
    );
  }

  #[test]
  fn transaction() {
    let test_server = TestServer::new();

    let coinbase_tx = test_server.bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].clone();
    let txid = coinbase_tx.txid();

    test_server.assert_response_regex(
      &format!("/tx/{txid}"),
      StatusCode::OK,
      &format!(
        ".*<title>Transaction {txid}</title>.*<h1>Transaction {txid}</h1>
<h2>Outputs</h2>
<ul class=monospace>
  <li>
    <a href=/output/9068a11b8769174363376b606af9a4b8b29dd7b13d013f4b0cbbd457db3c3ce5:0>
      9068a11b8769174363376b606af9a4b8b29dd7b13d013f4b0cbbd457db3c3ce5:0
    </a>
    <dl>
      <dt>value</dt><dd>5000000000</dd>
      <dt>script pubkey</dt><dd></dd>
    </dl>
  </li>
</ul>.*"
      ),
    );
  }

  #[test]
  fn detect_reorg() {
    let test_server = TestServer::new();

    test_server.bitcoin_rpc_server.mine_blocks(1);

    test_server.assert_response("/status", StatusCode::OK, "OK");

    test_server.bitcoin_rpc_server.invalidate_tip();
    test_server.bitcoin_rpc_server.mine_blocks(2);

    test_server.assert_response_regex("/status", StatusCode::OK, "Reorg detected.*");
  }
}
