use {
  self::{
    deserialize_from_str::DeserializeFromStr,
    error::{OptionExt, ServerError, ServerResult},
  },
  super::*,
  crate::page_config::PageConfig,
  crate::templates::{
    BlockHtml, ClockSvg, HomeHtml, InputHtml, InscriptionHtml, InscriptionsHtml, OutputHtml,
    PageContent, PageHtml, PreviewAudioHtml, PreviewImageHtml, PreviewPdfHtml, PreviewTextHtml,
    PreviewUnknownHtml, PreviewVideoHtml, RangeHtml, RareTxt, SatHtml, TransactionHtml,
  },
  axum::{
    body,
    extract::{Extension, Path, Query},
    headers::UserAgent,
    http::{header, HeaderMap, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router, TypedHeader,
    Json
  },
  axum_server::Handle,
  rust_embed::RustEmbed,
  rustls_acme::{
    acme::{LETS_ENCRYPT_PRODUCTION_DIRECTORY, LETS_ENCRYPT_STAGING_DIRECTORY},
    axum::AxumAcceptor,
    caches::DirCache,
    AcmeConfig,
  },
  std::{cmp::Ordering, str},
  tokio_stream::StreamExt,
  tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
  },
  reqwest,
  std::collections::HashMap,
};

mod error;

enum BlockQuery {
  Height(u64),
  Hash(BlockHash),
}

impl FromStr for BlockQuery {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(if s.len() == 64 {
      BlockQuery::Hash(s.parse()?)
    } else {
      BlockQuery::Height(s.parse()?)
    })
  }
}

impl From<reqwest::Error> for ServerError {
  fn from(err: reqwest::Error) -> Self {
    ServerError::NotFound(format!("Reqwest error: {}", err))
  }
}

enum SpawnConfig {
  Https(AxumAcceptor),
  Http,
  Redirect(String),
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

impl PageContent for StaticHtml {
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
  #[clap(long, help = "Redirect HTTP traffic to HTTPS.")]
  redirect_http_to_https: bool,
}

impl Server {
  pub(crate) fn run(self, options: Options, index: Arc<Index>, handle: Handle) -> Result {
    Runtime::new()?.block_on(async {
      let clone = index.clone();
      thread::spawn(move || loop {
        if let Err(error) = clone.update() {
          log::warn!("{error}");
        }
        thread::sleep(Duration::from_millis(5000));
      });

      let config = options.load_config()?;
      let acme_domains = self.acme_domains()?;

      let page_config = Arc::new(PageConfig {
        chain: options.chain(),
        domain: acme_domains.first().cloned(),
      });

      let router = Router::new()
        .route("/", get(Self::home))
        .route("/block-count", get(Self::block_count))
        .route("/block/:query", get(Self::block))
        .route("/bounties", get(Self::bounties))
        .route("/clock", get(Self::clock))
        .route("/content/:inscription_id", get(Self::content))
        .route("/faq", get(Self::faq))
        .route("/favicon.ico", get(Self::favicon))
        .route("/feed.xml", get(Self::feed))
        .route("/input/:block/:transaction/:input", get(Self::input))
        .route("/inscription/:inscription_id", get(Self::inscription))
        .route("/inscription_json/:inscription_id", get(Self::inscription_json))
        .route("/collection_json/:collection_id", get(Self::collection_json_id))
        .route("/collection_json", get(Self::collection_json))
        .route("/inscriptions", get(Self::inscriptions))
        .route("/inscriptions/:from", get(Self::inscriptions_from))
        .route("/install.sh", get(Self::install_script))
        .route("/ordinal/:sat", get(Self::ordinal))
        .route("/output/:output", get(Self::output))
        .route("/preview/:inscription_id", get(Self::preview))
        .route("/range/:start/:end", get(Self::range))
        .route("/rare.txt", get(Self::rare_txt))
        .route("/sat/:sat", get(Self::sat))
        .route("/search", get(Self::search_by_query))
        .route("/search/:query", get(Self::search_by_path))
        .route("/static/*path", get(Self::static_asset))
        .route("/status", get(Self::status))
        .route("/tx/:txid", get(Self::transaction))
        .layer(Extension(index))
        .layer(Extension(page_config))
        .layer(Extension(Arc::new(config)))
        .layer(SetResponseHeaderLayer::if_not_present(
          header::CONTENT_SECURITY_POLICY,
          HeaderValue::from_static("default-src 'self'"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
          header::STRICT_TRANSPORT_SECURITY,
          HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        ))
        .layer(
          CorsLayer::new()
            .allow_methods([http::Method::GET])
            .allow_origin(Any),
        )
        .layer(CompressionLayer::new());

      match (self.http_port(), self.https_port()) {
        (Some(http_port), None) => {
          self
            .spawn(router, handle, http_port, SpawnConfig::Http)?
            .await??
        }
        (None, Some(https_port)) => {
          self
            .spawn(
              router,
              handle,
              https_port,
              SpawnConfig::Https(self.acceptor(&options)?),
            )?
            .await??
        }
        (Some(http_port), Some(https_port)) => {
          let http_spawn_config = if self.redirect_http_to_https {
            SpawnConfig::Redirect(if https_port == 443 {
              format!("https://{}", acme_domains[0])
            } else {
              format!("https://{}:{https_port}", acme_domains[0])
            })
          } else {
            SpawnConfig::Http
          };

          let (http_result, https_result) = tokio::join!(
            self.spawn(router.clone(), handle.clone(), http_port, http_spawn_config)?,
            self.spawn(
              router,
              handle,
              https_port,
              SpawnConfig::Https(self.acceptor(&options)?),
            )?
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
    config: SpawnConfig,
  ) -> Result<task::JoinHandle<io::Result<()>>> {
    let addr = (self.address.as_str(), port)
      .to_socket_addrs()?
      .next()
      .ok_or_else(|| anyhow!("failed to get socket addrs"))?;

    if !integration_test() {
      eprintln!(
        "Listening on {}://{addr}",
        match config {
          SpawnConfig::Https(_) => "https",
          _ => "http",
        }
      );
    }

    Ok(tokio::spawn(async move {
      match config {
        SpawnConfig::Https(acceptor) => {
          axum_server::Server::bind(addr)
            .handle(handle)
            .acceptor(acceptor)
            .serve(router.into_make_service())
            .await
        }
        SpawnConfig::Redirect(destination) => {
          axum_server::Server::bind(addr)
            .handle(handle)
            .serve(
              Router::new()
                .fallback(Self::redirect_http_to_https)
                .layer(Extension(destination))
                .into_make_service(),
            )
            .await
        }
        SpawnConfig::Http => {
          axum_server::Server::bind(addr)
            .handle(handle)
            .serve(router.into_make_service())
            .await
        }
      }
    }))
  }

  fn acme_cache(acme_cache: Option<&PathBuf>, options: &Options) -> Result<PathBuf> {
    let acme_cache = if let Some(acme_cache) = acme_cache {
      acme_cache.clone()
    } else {
      options.data_dir()?.join("acme-cache")
    };

    Ok(acme_cache)
  }

  fn acme_domains(&self) -> Result<Vec<String>> {
    if !self.acme_domain.is_empty() {
      Ok(self.acme_domain.clone())
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
    let config = AcmeConfig::new(self.acme_domains()?)
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

  fn index_height(index: &Index) -> ServerResult<Height> {
    index.height()?.ok_or_not_found(|| "genesis block")
  }

  async fn clock(Extension(index): Extension<Arc<Index>>) -> ServerResult<Response> {
    Ok(
      (
        [(
          header::CONTENT_SECURITY_POLICY,
          HeaderValue::from_static("default-src 'unsafe-inline'"),
        )],
        ClockSvg::new(Self::index_height(&index)?),
      )
        .into_response(),
    )
  }

  async fn sat(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(DeserializeFromStr(sat)): Path<DeserializeFromStr<Sat>>,
  ) -> ServerResult<PageHtml<SatHtml>> {
    let satpoint = index.rare_sat_satpoint(sat)?;

    Ok(
      SatHtml {
        sat,
        satpoint,
        blocktime: index.blocktime(sat.height())?,
        inscription: index.get_inscription_id_by_sat(sat)?,
      }
      .page(page_config, index.has_sat_index()?),
    )
  }

  async fn ordinal(Path(sat): Path<String>) -> Redirect {
    Redirect::to(&format!("/sat/{sat}"))
  }

  async fn output(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(outpoint): Path<OutPoint>,
  ) -> ServerResult<PageHtml<OutputHtml>> {
    let list = if index.has_sat_index()? {
      index.list(outpoint)?
    } else {
      None
    };

    let output = if outpoint == OutPoint::null() {
      let mut value = 0;

      if let Some(List::Unspent(ranges)) = &list {
        for (start, end) in ranges {
          value += end - start;
        }
      }

      TxOut {
        value,
        script_pubkey: Script::new(),
      }
    } else {
      index
        .get_transaction(outpoint.txid)?
        .ok_or_not_found(|| format!("output {outpoint}"))?
        .output
        .into_iter()
        .nth(outpoint.vout as usize)
        .ok_or_not_found(|| format!("output {outpoint}"))?
    };

    let inscriptions = index.get_inscriptions_on_output(outpoint)?;

    Ok(
      OutputHtml {
        outpoint,
        inscriptions,
        list,
        chain: page_config.chain,
        output,
      }
      .page(page_config, index.has_sat_index()?),
    )
  }

  async fn range(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path((DeserializeFromStr(start), DeserializeFromStr(end))): Path<(
      DeserializeFromStr<Sat>,
      DeserializeFromStr<Sat>,
    )>,
  ) -> ServerResult<PageHtml<RangeHtml>> {
    match start.cmp(&end) {
      Ordering::Equal => Err(ServerError::BadRequest("empty range".to_string())),
      Ordering::Greater => Err(ServerError::BadRequest(
        "range start greater than range end".to_string(),
      )),
      Ordering::Less => Ok(RangeHtml { start, end }.page(page_config, index.has_sat_index()?)),
    }
  }

  async fn rare_txt(Extension(index): Extension<Arc<Index>>) -> ServerResult<RareTxt> {
    Ok(RareTxt(index.rare_sat_satpoints()?.ok_or_else(|| {
      ServerError::NotFound(
        "tracking rare sats requires index created with `--index-sats` flag".into(),
      )
    })?))
  }

  async fn home(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
  ) -> ServerResult<PageHtml<HomeHtml>> {
    Ok(
      HomeHtml::new(index.blocks(100)?, index.get_homepage_inscriptions()?)
        .page(page_config, index.has_sat_index()?),
    )
  }

  async fn install_script() -> Redirect {
    Redirect::to("https://raw.githubusercontent.com/casey/ord/master/install.sh")
  }

  async fn block(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(DeserializeFromStr(query)): Path<DeserializeFromStr<BlockQuery>>,
  ) -> ServerResult<PageHtml<BlockHtml>> {
    let (block, height) = match query {
      BlockQuery::Height(height) => {
        let block = index
          .get_block_by_height(height)?
          .ok_or_not_found(|| format!("block {height}"))?;

        (block, height)
      }
      BlockQuery::Hash(hash) => {
        let info = index
          .block_header_info(hash)?
          .ok_or_not_found(|| format!("block {hash}"))?;

        let block = index
          .get_block_by_hash(hash)?
          .ok_or_not_found(|| format!("block {hash}"))?;

        (block, info.height as u64)
      }
    };

    Ok(
      BlockHtml::new(block, Height(height), Self::index_height(&index)?)
        .page(page_config, index.has_sat_index()?),
    )
  }

  async fn transaction(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(txid): Path<Txid>,
  ) -> ServerResult<PageHtml<TransactionHtml>> {
    let inscription = index.get_inscription_by_id(txid.into())?;

    let blockhash = index.get_transaction_blockhash(txid)?;

    Ok(
      TransactionHtml::new(
        index
          .get_transaction(txid)?
          .ok_or_not_found(|| format!("transaction {txid}"))?,
        blockhash,
        inscription.map(|_| txid.into()),
        page_config.chain,
      )
      .page(page_config, index.has_sat_index()?),
    )
  }

  async fn status(Extension(index): Extension<Arc<Index>>) -> (StatusCode, &'static str) {
    if index.is_reorged() {
      (
        StatusCode::OK,
        "reorg detected, please rebuild the database.",
      )
    } else {
      (
        StatusCode::OK,
        StatusCode::OK.canonical_reason().unwrap_or_default(),
      )
    }
  }

  async fn search_by_query(
    Extension(index): Extension<Arc<Index>>,
    Query(search): Query<Search>,
  ) -> ServerResult<Redirect> {
    Self::search(&index, &search.query).await
  }

  async fn search_by_path(
    Extension(index): Extension<Arc<Index>>,
    Path(search): Path<Search>,
  ) -> ServerResult<Redirect> {
    Self::search(&index, &search.query).await
  }

  async fn search(index: &Index, query: &str) -> ServerResult<Redirect> {
    Self::search_inner(index, query)
  }

  fn search_inner(index: &Index, query: &str) -> ServerResult<Redirect> {
    lazy_static! {
      static ref HASH: Regex = Regex::new(r"^[[:xdigit:]]{64}$").unwrap();
      static ref OUTPOINT: Regex = Regex::new(r"^[[:xdigit:]]{64}:\d+$").unwrap();
      static ref INSCRIPTION_ID: Regex = Regex::new(r"^[[:xdigit:]]{64}i\d+$").unwrap();
    }

    let query = query.trim();

    if HASH.is_match(query) {
      if index.block_header(query.parse().unwrap())?.is_some() {
        Ok(Redirect::to(&format!("/block/{query}")))
      } else {
        Ok(Redirect::to(&format!("/tx/{query}")))
      }
    } else if OUTPOINT.is_match(query) {
      Ok(Redirect::to(&format!("/output/{query}")))
    } else if INSCRIPTION_ID.is_match(query) {
      Ok(Redirect::to(&format!("/inscription/{query}")))
    } else {
      Ok(Redirect::to(&format!("/sat/{query}")))
    }
  }

  async fn favicon(user_agent: Option<TypedHeader<UserAgent>>) -> ServerResult<Response> {
    if user_agent
      .map(|user_agent| {
        user_agent.as_str().contains("Safari/")
          && !user_agent.as_str().contains("Chrome/")
          && !user_agent.as_str().contains("Chromium/")
      })
      .unwrap_or_default()
    {
      Ok(
        Self::static_asset(Path("/favicon.png".to_string()))
          .await
          .into_response(),
      )
    } else {
      Ok(
        (
          [(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'unsafe-inline'"),
          )],
          Self::static_asset(Path("/favicon.svg".to_string())).await?,
        )
          .into_response(),
      )
    }
  }

  async fn feed(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
  ) -> ServerResult<Response> {
    let mut builder = rss::ChannelBuilder::default();

    let chain = page_config.chain;
    match chain {
      Chain::Mainnet => builder.title("Inscriptions"),
      _ => builder.title(format!("Inscriptions – {chain:?}")),
    };

    builder.generator(Some("ord".to_string()));

    for (number, id) in index.get_feed_inscriptions(300)? {
      builder.item(
        rss::ItemBuilder::default()
          .title(format!("Inscription {number}"))
          .link(format!("/inscription/{id}"))
          .guid(Some(rss::Guid {
            value: format!("/inscription/{id}"),
            permalink: true,
          }))
          .build(),
      );
    }

    Ok(
      (
        [
          (header::CONTENT_TYPE, "application/rss+xml"),
          (
            header::CONTENT_SECURITY_POLICY,
            "default-src 'unsafe-inline'",
          ),
        ],
        builder.build().to_string(),
      )
        .into_response(),
    )
  }

  async fn static_asset(Path(path): Path<String>) -> ServerResult<Response> {
    let content = StaticAssets::get(if let Some(stripped) = path.strip_prefix('/') {
      stripped
    } else {
      &path
    })
    .ok_or_not_found(|| format!("asset {path}"))?;
    let body = body::boxed(body::Full::from(content.data));
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Ok(
      Response::builder()
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(body)
        .unwrap(),
    )
  }

  async fn block_count(Extension(index): Extension<Arc<Index>>) -> ServerResult<String> {
    Ok(index.block_count()?.to_string())
  }

  async fn input(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(path): Path<(u64, usize, usize)>,
  ) -> Result<PageHtml<InputHtml>, ServerError> {
    let not_found = || format!("input /{}/{}/{}", path.0, path.1, path.2);

    let block = index
      .get_block_by_height(path.0)?
      .ok_or_not_found(not_found)?;

    let transaction = block
      .txdata
      .into_iter()
      .nth(path.1)
      .ok_or_not_found(not_found)?;

    let input = transaction
      .input
      .into_iter()
      .nth(path.2)
      .ok_or_not_found(not_found)?;

    Ok(InputHtml { path, input }.page(page_config, index.has_sat_index()?))
  }

  async fn faq() -> Redirect {
    Redirect::to("https://docs.ordinals.com/faq/")
  }

  async fn bounties() -> Redirect {
    Redirect::to("https://docs.ordinals.com/bounty/")
  }

  async fn content(
    Extension(index): Extension<Arc<Index>>,
    Extension(config): Extension<Arc<Config>>,
    Path(inscription_id): Path<InscriptionId>,
  ) -> ServerResult<Response> {
    if config.is_hidden(inscription_id) {
      return Ok(PreviewUnknownHtml.into_response());
    }

    let inscription = index
      .get_inscription_by_id(inscription_id)?
      .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

    Ok(
      Self::content_response(inscription)
        .ok_or_not_found(|| format!("inscription {inscription_id} content"))?
        .into_response(),
    )
  }

  fn content_response(inscription: Inscription) -> Option<(HeaderMap, Vec<u8>)> {
    let mut headers = HeaderMap::new();

    headers.insert(
      header::CONTENT_TYPE,
      inscription
        .content_type()
        .unwrap_or("application/octet-stream")
        .parse()
        .unwrap(),
    );
    headers.insert(
      header::CONTENT_SECURITY_POLICY,
      HeaderValue::from_static("default-src 'unsafe-eval' 'unsafe-inline' data:"),
    );
    headers.insert(
      header::CACHE_CONTROL,
      HeaderValue::from_static("max-age=31536000, immutable"),
    );

    Some((headers, inscription.into_body()?))
  }

  async fn preview(
    Extension(index): Extension<Arc<Index>>,
    Extension(config): Extension<Arc<Config>>,
    Path(inscription_id): Path<InscriptionId>,
  ) -> ServerResult<Response> {
    if config.is_hidden(inscription_id) {
      return Ok(PreviewUnknownHtml.into_response());
    }

    let inscription = index
      .get_inscription_by_id(inscription_id)?
      .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

    return match inscription.media() {
      Media::Audio => Ok(PreviewAudioHtml { inscription_id }.into_response()),
      Media::Iframe => Ok(
        Self::content_response(inscription)
          .ok_or_not_found(|| format!("inscription {inscription_id} content"))?
          .into_response(),
      ),
      Media::Image => Ok(
        (
          [(
            header::CONTENT_SECURITY_POLICY,
            "default-src 'self' 'unsafe-inline'",
          )],
          PreviewImageHtml { inscription_id },
        )
          .into_response(),
      ),
      Media::Pdf => Ok(
        (
          [(
            header::CONTENT_SECURITY_POLICY,
            "script-src-elem 'self' https://cdn.jsdelivr.net",
          )],
          PreviewPdfHtml { inscription_id },
        )
          .into_response(),
      ),
      Media::Text => {
        let content = inscription
          .body()
          .ok_or_not_found(|| format!("inscription {inscription_id} content"))?;
        Ok(
          PreviewTextHtml {
            text: str::from_utf8(content)
              .map_err(|err| anyhow!("Failed to decode {inscription_id} text: {err}"))?,
          }
          .into_response(),
        )
      }
      Media::Unknown => Ok(PreviewUnknownHtml.into_response()),
      Media::Video => Ok(PreviewVideoHtml { inscription_id }.into_response()),
    };
  }

  async fn inscription_json(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(inscription_id): Path<InscriptionId>,
) -> ServerResult<Json<serde_json::Value>> {
    let entry = index
        .get_inscription_entry(inscription_id)?
        .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

    let inscription = index
        .get_inscription_by_id(inscription_id)?
        .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

    let satpoint = index
        .get_inscription_satpoint_by_id(inscription_id)?
        .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

    let output = index
        .get_transaction(satpoint.outpoint.txid)?
        .ok_or_not_found(|| format!("inscription {inscription_id} current transaction"))?
        .output
        .into_iter()
        .nth(satpoint.outpoint.vout.try_into().unwrap())
        .ok_or_not_found(|| format!("inscription {inscription_id} current transaction output"))?;

    let previous = if let Some(previous) = entry.number.checked_sub(1) {
        Some(
            index
                .get_inscription_id_by_inscription_number(previous)?
                .ok_or_not_found(|| format!("inscription {previous}"))?,
        )
    } else {
        None
    };

    let next = index.get_inscription_id_by_inscription_number(entry.number + 1)?;

    let content = format!("{}{}", "/content/", inscription_id);
    let preview = format!("{}{}", "/preview/", inscription_id);
    let metadata = inscription.metadata();
    let result = serde_json::json!({
        "chain": page_config.chain,
        "genesis_fee": entry.fee,
        "genesis_height": entry.height,
        "inscription_id": inscription_id,
        "next": next,
        "number": entry.number,
        "output": output,
        "previous": previous,
        "sat": entry.sat,
        "satpoint": satpoint,
        "content": content,
        "preview": preview,
        "traits": metadata,
        "collection_name":"People of Eden",
        "collection_author":"MahaDAO",
        "collection_description":"The People of Eden is first of its kind DeFi PFP collection combining art, robust utility, and storytelling to communicate the importance of financial freedom through an open yet decentralized ecosystem using $MAHA and $ARTH.",
    });

    let data = serde_json::json!({"data":result});

    Ok(Json(data))
  }

  async fn collection_json_id(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(collection_id): Path<String>,
) -> ServerResult<Json<serde_json::Value>> {


  if collection_id != "2e45df" {
    return Err(ServerError::BadRequest("Invalid collection ID".into()));
  }
  

  // let ins_id = "0f0226818f3d8b4a3d1c4cb1d5e9c8ddce83603a16b365a960b6ef9576ec1b93i0";

  // //let data = Self::inscription_json(Extension(page_config), Extension(index),ins_id);
  
  // let base_url = "https://api.ordrare.org";
  // let relative_url = "/inscription_json/";
  // let url = format!("{}{}{}", base_url, relative_url,ins_id);

  // let client = reqwest::Client::builder()
  // .timeout(std::time::Duration::from_secs(10))
  // .build()?;

  // let inscription_response = client.get(&url).send().await?;
  // let inscription_res_json = inscription_response.json::<serde_json::Value>().await?;
  // // log::warn!("{}",inscription_res_json);
  // // print!("{}",inscription_res_json);
  
  // let inscriptions =[inscription_res_json];

    

    let ins_ids = [
    "7331f9bbed073f987dbf1967f0ce58970c50a4acb7cc5ac033db84ee1f76e600i0",
    "79f14f5467c5ed92b28d44ed8c5469239cbe89ee88534b71e668f829303c5501i0",
    "e77c2fbec4640430cab6e22756c92f84f5673bce17ab2b31b88912e00bcfc501i0",
    "d341f3799e6ca24c419f9a18377a1d1583e6b3de7d3266e477f15957305be401i0",
    "aa0265e61b0e609027bb2393d61e6f3a6030b32e82a8ca75b0c827dadfca5202i0",
    "a554fc88282b907e28aab3fda93f017989fefee111d9f5e09ab1b507dd4c8a04i0",
    "0cfef41200884a2f39a72c0500f610edc0e51856c9625291f3b3776f74136d0ai0",
    "eaec9871210aaefbc9bc7db513804149f55bd5e7fec83612083b0028cf97c40ai0",
    "79f6703acc34b324fb625093567070d26d53c69682d9eeb1296a9892467c880di0",
    "87c7086449f46a7fc38402277bf2fcf5a9d4a3c84547fcd80592518dd7b0cd0di0",
    "5478b3b4577000871bb2863d2bb048015fba4ab75910c0d8c81f93d405e4c60ei0",
    "fae2847d2fd3b18ba5044f7911bc99c3932ef44f7a2ebbeaf2b630454685fa0fi0",
    "1e130fa90c44961d3fec458f1a8ecba9f26fb5f4dd89a821c005c8fa609ae811i0",
    "59e454dba2c42d7f34733f739db7fe88ba866dc9bfcc03f4b5af8fa328ed6812i0",
    "e303d5a619a1682282737082c25c6b0df8eede536f904ea48164cdb26053d912i0",
    "8d3ac1d83b661c6b42b984cd7e93293cecaff4594d25d399f84522adf4ba4e13i0",
    "b70d967abb0fd98db906b74201ca08c341c4a02913c4047bcd0013e86a9de014i0",
    "c8a991de0341a5865f74fc389d74d9568a2a5ad2964019e4462b8ec437505515i0",
    "02d50a30e7e92dfb380187174fd17cc9079aecfc399728f4086b8ec8f0b7fe17i0",
    "88170866d782d021b53671c5214288bb1aa627f9da78e2fb8e44f0babf9fb018i0",
    "4d7c95e35396b06eaf461e76992866a7e48d94ed810758b67d5fd2076eed111ai0",
    "4c57442ab512ec4ace17f8ac7720cb72f56af0a544a14c2e6fb0ec851524141ei0",
    "89a078660b53527e170b9adaa5520c98b7e90cf3e0bcf6ae6ee0324f7ce4e320i0",
    "7cc8c970e28bed8381455702b86e395d9a432a7b460e0650ffd2fff1a3790e21i0",
    "bc22ba231b110ea99945ccc5002ea14ec674b8ad11ba92a0b2df0028ca9acc21i0",
    "636dc4ab0a181fb1b40fe0ee6ffd7bdc7d713ae0054a229e8b1feff46fd55723i0",
    "f6d69a881ff287f8be3d4508ecf14db97567b95a10a41aec425a4cf3e92de223i0",
    "1b9e9a3a919bcc32b3c2cb28746ad0a98bc691c03e22461c8b6ea5289da0d724i0",
    "a781a793ff1de32ec01569a8085b343c653ef2ce1029ee5ba347bac7c505ea25i0",
    "92e1f714f3e58ee003eff9a8cd1cb62f91f79cddf1c6c8da4dd1af5836f4d226i0",
    "ea6936b5d61ec76c556e88ab325dd3e4e302662980e3a3465fc37c03c9dd4827i0",
    "929ecc0c8e0961722b3fff4303f0e9fb0534e04f58a4579d4923095718fbee28i0",
    "43386c448d525457a05a33c770a294fdbc0296d378aa803b160b3315e9288529i0",
    "316b0e7b37cefe1c1afc972479fc29fc7f55e26659a858ec23dbe529158fc029i0",
    "a256ea0f0686b4780b0d8a07a5ed617dce724b1d37b35830d7e881a886ec8b2bi0",
    "4851fa2f09aa01a9a6d976de97bfb2880b3b41dff887d4d413de290c4d09982bi0",
    "3be3e8cc21ec085e861266a65b4e6a2884bbc04aa78e6eb3328e792a2f52c62bi0",
    "75df937d8a9fec3f865d973f868aab84c7d3de7b83431bbbdc9ecc3d0ebc2b2ci0",
    "6079f243b8b14e632b7729b78a3bdcf6643fe7a5c7bdde01687c8e2236028e2ci0",
    "1f7ba156fdee9fdefbb95dc1a642c77a3d1e6c843f563299bdb838dfda7ec92fi0",
    "fa7a961c9766360cdfdbd6b89d99ff8351d56c56c478b433ac3032702762a233i0",
    "49e7d8a0d4744c3683ea301ed89781f5b1fde4973e8040f0deda50eee51dd133i0",
    "9925a0fd4872b9cf23ddad57563f181fb909229e581518e90b5cd80a0697dc33i0",
    "b4f47009951706b79ca98f978b89f6428b674c49d487e8ac9ddb833fee0b3734i0",
    "0630b591a5e557a1ed8304073dfcea6f4423ea33f579f738d6e8fe37d55d6635i0",
    "8eb59ee9803d1ab7c8465348de014c42813ab396e7a98f576075d10ad68ca135i0",
    "3aba6cc7b18bf71253020c3474e3b8591dd78ea2b0c14c69c7a79069aaa64d37i0",
    "5092ddec28a2bc96238ae8eb3e99118e0aca4a121b6a8351c818dfa601ec9139i0",
    "7d8a728e7a42412f68ef5c40ade1a1976b11a825252c9f380f9dd77126d7f839i0",
    "64aa28fcb8e9802967a0b66c068046aa57e154c21769c0d3611220274225733bi0",
    "ca70ee980690792b2bb2a1b4dcb4910260d2947cb646c3282a61132fbf73a63ci0",
    "a71ea3188bbc3fb3734901520dd6567a0095c6c4c1f44198b41831e1bbabce3ci0",
    "4d3d028855babe4e2ea4861121bec8eb2bb5a41f13b4b30e61dd8bcd9978133di0",
    "0f3c5584030a06e0fdd16e67c4cc631445f76a003f76860b27f612894fc20c3ei0",
    "b1b4edcecee5aaa8ae5631aaa06bf8594bf8008571c9021c7b8cfc4bdc585c3ei0",
    "4e3c1937eb43d8b4a4ee341c2a4ddbaabcabc461d66475d285a367e6ad67f13fi0",
    "c1b22c0edb4476c753c230fb7804b4568261f6328d4d1aa34313160a637e2b40i0",
    "02570bf2bdada212ae745a121545a97e5be7d6b5c1d63332973a2f41309bf144i0",
    "681d2bf14680d88f817cd7d3bf6160abd68953b5bc1229a0ff184b54c5219546i0",
    "7750686f22a660418e5821f54ba24714c240ea7039b4be48b3a9ab550baf3d47i0",
    "18b778270a7e394866a4286f2d805b61c99dc798d202163e3c3aca7f46467248i0",
    "23e0fe730884b7f4ccc3780c4adf7701b5fcd899947a1a4f6bf051053aa10e49i0",
    "eadfb4ecb5f0758b5e6fd3e53aac27b1e2b1ec28b9d5c710c8d8416db1ca394ai0",
    "241ba21ea038bf777122be94f5a7e78cf2747962562dfba832d35240192b634ci0",
    "7878dbe04eeb4b0bf850d0e3abac70a1b79093bb471a04569d566056a6a1764ci0",
    "09de9e06634ac9c34492387c2ba6ceaae9591329f427a2772cd15d2e45424e50i0",
    "7ad46cca0637291eeefacc7f1b68265a23845d349b7fe9fd279704e485c06f58i0",
    "11ef23e2311fc204c483c21ca0f53aad46d94cc708be1ce21066f360ef6e7958i0",
    "09612feb9929c191d7e66cf30451c21a64707353997efe13bcd5840dea86cd5bi0",
    "df49292c0d11e03c66df194761ac8b70494505e89ba96bfe56b86cb42f45715di0",
    "579d0c11fcd1c5b1ed81b3bc77f758da24eb08e78275b51e9e1ea1a9d98bc25fi0",
    "6b65d5bf54df38d57af9020c4d927bcc193a1c8848a8aa7b199163b2c1541f60i0",
    "42dcbeb126bb8f4b1264775630940cd34d88d08c3c372a0685de230bd8265d60i0",
    "ad252ae6e7b30330e08c25c285164884bf48ddb3fde71e6289d711f96fd9ca60i0",
    "4acd9b62d578cc656ad9c5e71f4f0b5e3c3b05149102798de98e6303324cfa60i0",
    "4e8f6c5b864457b4ba9f86580dbf273d947801bfc5e99bdb9c732707ede70164i0",
    "43a130e19797217e1bab064a1db965528f459f55e280d6c9881881ca5d993c66i0",
    "295b377e28b2bcf0b62c4e6bc647e5e05b93966c246985b940e31f359f1b9c67i0",
    "d883f1b41b5c168c2f035d385f0fd609b863d55670b3d832ea2fada62543c067i0",
    "adeb1761906e189d6ce64400adc26cdcc813dd8aaaa0b330f1a2be670105f667i0",
    "d97d157a9c686a49c50fcec47c32185ff030515c6b15f9fdff5d24ab1a504d69i0",
    "ab25e70ffb4f561d863133f46f878565d0c526983ca70139ebfb8420eb90676ai0",
    "2b608630b0d049250beaea5d743191d93228d6db84d3d189e4bf3b20b5d8b76ci0",
    "0b08c3fe765a53a62f435b06796e9a5d90e2c6e3e02a9ae857edbdbf56f3736ei0",
    "079b95f0570067cfbdcd7c9d0cd05c8fc039c688cdef106830d5e688bb6b3171i0",
    "aa16e7e1b2540c913c3346b01b3d7897e804e7ec0f17345a78c0c6be4f85f972i0",
    "a897452508fc5d9c0ef4290d0b864362e7ba02a73d7495781ef84f07ff28fe72i0",
    "c9798530ed2695cc4e6a89bc088973f716d552cbb6c0fe933f33d7c223507f73i0",
    "1f63e0ec8efa75d9cabb09e28943c819c4b3569d040103c26b521344d3340874i0",
    "6c6e291709c25e752b0e59366dc9542e4e1d675793071f7fea86be0b46cb0775i0",
    "fd16ef38a7eabee1b69c761b29b433b381becfc0469172953c50b1be72cb0e75i0",
    "1dbf3b9508041308064b5ffc1a92fcd129657c38e2c627628061e8299ff5b475i0",
    "269f8f376953a2644721d538569b7c6f35497c44ef310ff9faf6803f8e137376i0",
    "4c4ff306743dc20fd2331befb3746978c33fb38f3a7000eaee2ecc1fea6f6277i0",
    "751a2baa92777a0458796a835dc856e77bb55a9ef978db83915d6c4aa59d9379i0",
    "f2fb60f4ed762d586fdf1567b9d5ee5adfdba589249b94581ed9499b1c6da87ai0",
    "a886b97d239ef83a4f890ae3b2ade15b47f1e4217fc3b1238e2eb4ab01a7037ci0",
    "d4bcf73adaef590bba1be494b67eebef9b0bd0d668086500402205e0b238357ci0",
    "2492223047640fc140e471f825da2689cbddd0a525f753122fb933291dd1be7ci0",
    "ebe2bd79048afb762a02f3f21e615e441d91a0bb0fcb08611bb2130f8612697ei0",
    "0d053ba2543c2f0883ece4b17ca860f8595ea8ac35c132d2e42b8dc3d9a98381i0",
    "b6861b89015a6cd5df2d66226b4f589901b690cdbd0cab1dce679b29cb4eba81i0",
    "a9befc1374f93779e566d7bc5fc268a7e09cd9430640f64cf742f13fb87ac481i0",
    "8dc7aa6c0f516bfa094d560daf108c88696e33568664548948d6789905455282i0",
    "cda09cededbbb4747703f86fe199a1c7e09bff85af1bfaea72a0073d1df8c383i0",
    "df16c4dca28152ffefc6ef07fa1a1fdea3e629f2dc1d767a67c7e0e7fb9d0384i0",
    "dd623de1765a1572d88ddb7054ea20c4de2c32800f172082f5cb2f08ff121e84i0",
    "98cd26f6420fa22bf6c40d91b7bea3cb27118aa805647cb641607deeb2712884i0",
    "8f2757cbc215b1f7d1608fa00eb06d35eb6c2e648cfd5f7dd1a54eb25abd8787i0",
    "65a1b2bb49e2e8c2619cbef0b241367de7060a2839fd23f20260bbb9c7649a88i0",
    "b3d3f1acc772da2f7eab3d51d2705071cdbe05dbe524f4f336da936afb0f6c8ai0",
    "ed3d0933477be8221b744684350f78e2e51a77ff4bc7197ae01c78e786a6718ai0",
    "c3b64c5a7217a546178899962f0064fb49331bb6243e376896d9a2033ed5b38ai0",
    "3109bf23cb29d44762d108263b099923e30f57b9c402dabf442804b704f87d8bi0",
    "a67f327337a4b90358beaaf457ffc27bce81785683622df6ec879bc635e74190i0",
    "3aff8b929b537c725cebef9d3cba63b8d8a8864b6cd175f00635564130847592i0",
    "58ee262f1a96ce49c8368f8dcb6f99fc4b2940a7c9c789c8230cc1f46a62d692i0",
    "1321d4cb7edc52682497141d988539b461dd5aecb52adf7eb2819b90bc1f8d93i0",
    "2fb2fd13f60041413f3ae63177c251c9176cd0d0635cf8bcc56550ecf66f8494i0",
    "b6efc59f4da8e7566066a9e45ea1200aca8c242b2ec4e9d0996e6ccb89ecbe95i0",
    "8dc669ae56a65f0988510af0ee0d6d9d069b954b56fc8c8a7e4bb2cd565e3b98i0",
    "6bc218d915bbba60b762cc842d96244f4c845c5760ce63004a2f7032ff2bee99i0",
    "1333a2e5868386c8ad94561b8e10c6800bc7e51c9a07ffcb7902dba45831e29ci0",
    "4936ab7a37c70d1f1a0de6bf698bfe2552853b40ad5da645d8f957d1d846569fi0",
    "437f174449599fc822649106378a1ace9eec0b15353d2dd0c36cffba30c1b59fi0",
    "b025503d4a93c865d2f4b20b4565a75e5adf5c0e53cfd79166af2cd2590e29a0i0",
    "2b1b726a0b2bc68ac2c66f497c619906822be22ae4fd9487ac8dfefbf332b0a0i0",
    "10b2a00dc9a9ad62d1b420ab43fb41f33f8e66f880af66f3223fae59b1ffc4a2i0",
    "5934aa852c1f8cf334dd14495008a30d5f1d88a89b6a1c46fcd29966d6da22a3i0",
    "61f289de8a3c1d9ee60804a839df0fafcd30031b50dbdc785ee96ab31509dca3i0",
    "2abaf32189070c9c8d9af58023e88703c1e905dd10b138ad8c27696bc7ce5fa4i0",
    "0fe40eee46311f5179dba89e8a81fe2d1b9ef3d9a988e27ca4025a449f8c72a5i0",
    "5fb8fb63479ea461f9d24c3dba26c47929474d90b3b98e0f0d368d8f1923aaa6i0",
    "019ae1e71d299769d5e6048f842274d500156da4e10520af14b9640b396311a7i0",
    "6bfb4b3953f6921d76908231d1d193d8a915feb227350f494402194e035bcca7i0",
    "ef4e142da4e29296f0ea3b9104f5e22a38b838419a641e52f103fb9ca8ca38a8i0",
    "bb8ffded44aafcbb7a775033238c861d8d461c44c7d1c3b206c4f9016d6657a8i0",
    "401194b561284bcdd29a34d7f70094473237141bd8beccd3b32e5ba91e17aaa8i0",
    "d435315308c20952802352a24ef2c1a7c54f2e1c0233acf5e29c2b47ecf23daai0",
    "d5932237a8e57182abb5e885454f4f74cd9f7d52eec686f59426d425ffd844aai0",
    "011876d54f2b8ef79aee1566fde47e6fc8b5c7a18b4fe84dcd9ca6d9918bf3abi0",
    "d9684a6ec65b45e91a1f5989958bd7148d04cd8dd013849c2e80ebbc66e9fcabi0",
    "cbe08a8df1da29082602c6edb24e06a0ea43f9bc0697b94b9314788c6ad1dfaci0",
    "aea82208fe5365e49e65454e771f3065b604d1de5e9f53afae29ea2d36734db0i0",
    "c67af500dbe106335fcdf6caf7c72ac9d811d81dacd5b9cb5229b43db9dc4cb2i0",
    "b1ec3f8054d024ae1edb2de6936f49982ce3273056047c979bd18af3f9dc6cb5i0",
    "82387692c103432d017de778281941f3846406bb4e6fa61894e0c45ba60dd2b5i0",
    "a8b264f6b7536762ef70e71c3e850c2e952e30c17730e5234c112abf6da50fb6i0",
    "cb74ed9927577784cc3ca16f37fd0ec9c51ef5c2f7e8fb45c96d15aaf5f49db6i0",
    "882507b08906ddffe3024815f460d6bd2a64a570584cf0c6a18ebd575019afb6i0",
    "a258fa86e2277b18954164aa3085147ed779d7c4533376740f5b9cef11f9cfb7i0",
    "85683f039d7d4351a82212b213f66568106b8d0b135499dbd19f5c6e27f11cb8i0",
    "d3621404ebd5764d66b0558eff42772b20369bad0c39b00979b29c6251feaab8i0",
    "6bd1eb5cc60a807ac34087d40b9ec8c53bd2f3236c80ba33a3811d23ec0075bdi0",
    "f75ded88a91e9e757ee27aeaa6324cef09a42ee6dba6b5585ac6da21791ac1c0i0",
    "e318cc4b253e2170a9bc07bd6476a76f994012ccabb91c838eba13adf5e3fbc1i0",
    "a99a3df1d1821a3caed80cf6f8bb1c6f8725680a70d6b85850e018b063fee8c2i0",
    "573b7e039d4e7d39f02360849fc7bbe3e15c337d69a9a7fd28f72762104048c3i0",
    "f049d87acab62236538c80f7c2daa49664a71e0b75e805b15e9e6c2a8d963bc8i0",
    "237b7b291e7d632b0f0774036762e59901b97275144e0fd38013bdeb88b67ec8i0",
    "4d73e1dfa10a823fc9841fe770d372d26262b45c80530c6cfd30eeb28bb433c9i0",
    "5eab18fb6ae301f1aa9b672a668884043e1930a6f4eccfca581299971cadbfcai0",
    "a3fc69334389e7054e9bd2efcacad15b3f5890bb1af78f5c8099c74f8c30d8cbi0",
    "54bd72c6a5a4b27de30a41e43bb953a7b0171f2b0b43d788675046837eb5f0cci0",
    "8df0426fad1eb4436a7da881e39606d79323f78ec5c2e9039bf04c2c925f8ecei0",
    "f2a9465aa7900648dc83e358b941dcec25d943657b8e7a54cb2a0807c4bbd5cei0",
    "0e3e84e7f92080edfc56a7ffd4649269d539d20fadf1007c9cec9960660923cfi0",
    "672db83dc896364d7180f8c71a1816b122d4016c0d50478ddbace363b09dabcfi0",
    "66fab1ed875ac28e32decaa2d4abf0de50ea913b235a4cb4e1c60908f2af27d2i0",
    "b1b320449674ee59d059b2c1e062173ee0432fe438c907c54fd52818af9a90d2i0",
    "c0aec76ae7e4f0bed3b4effb8fb1db5f5bf8de7f3633925933f4c3c3119704d3i0",
    "8f4ce60dbc8f82a628268b74b6465b7947f10e8288a113753fca80cfc53cc5d3i0",
    "bd9cf70f06b54eab0719016b02ddfc543eeb3d22bdd139ce0f7261ce53415bd4i0",
    "8eed8dfa6efa3f86bacb3a48267953e784fb01ac6fab17b5b1b01a6bfda17ad5i0",
    "a0fc670979c7a2564f0b22b9a255ef943df1af4e96293baac0088f5550ec0fd6i0",
    "7bc29112f2ea4e03bc5b3ba90ba2ad5417ed714175688a292e80a39f41b058d7i0",
    "e998cd4ec8ce5bbede903a50722310547380af949b532e0f6f4432eece7325dai0",
    "38883a8c7e4462538b21677397bdd27f15d0d86c31b82b162eb64d152384cedbi0",
    "82ce56f05561f65c68864e55689e9fb660df9d8412a43c6f159dfc6a5bccd0dbi0",
    "c4566dc6148d9ff28dd8a617e5a3e07e3bf237f380b0210337277336b76cf5dei0",
    "be7c0de84b6cbff5304ff589823e597c66898b14a7f2d3a5629f5405d0827bdfi0",
    "5982f4f3867affc8db2b20f3eb835269101e3d223f4162a46b7dc39897ff1be0i0",
    "1853cad718449fc5c2ffda856912bc7b0e71f8238fcff2b2a59a01a4c1c112e1i0",
    "621a6d6d382387b6e7bd70b72b92d556ce7f270ae7d26b57a3403dcf3c24e2e1i0",
    "9fc38ca2c4745cfd224a3ca1d89ae4d34cc2fa83e46016c33c6819748b53e1e2i0",
    "a33293ad001eb26ccc05bace4874422c8796b3da0f2c5e5811d6c15e11dc01e6i0",
    "d1dac27b32e6f5a7fd0a6abd127d041260b174d85abc68ca7a8a7f11f2b69ce8i0",
    "c35f0635ef5455c34f1590e4963ff0c772458465d3cf990116d563cb4af0f4e8i0",
    "bee5a3cbbb45d5c1fb3e4e4682af6789713d5d1be4bac0e1bb755741ddfef9eai0",
    "bcdb80f550e76da18612fa15d87a55580e772e2d939e6d3f66da1f03c03446ebi0",
    "c808749bd73724f71ddf938571cb926b5cb43e4f217588e83fdea4d87cb063eei0",
    "4c6873c40c934743e835235c1c557ca28a9c4618b00151335ab2858c7f6f01efi0",
    "9e0d0a3dcacbc570ac69aa6b52be1bdc9a3cdcbb2ac411593bf3c0cff7d10fefi0",
    "13d95a8306c8f651aef4f60f9a6cacb2e2020698aad184c61328d2f0f6372befi0",
    "aefb3022e8d74000293f685e7f38039e68e3358a809146fab5f1797d4e5ee1f0i0",
    "821788963f83926b123f048fcaa4c2dae78a41cc4bbfb60373aff15bc71539f1i0",
    "ad18b315ce1ff39f0f2a49ee61d8ed925619521227be5aea14ce4b039e4800f4i0",
    "33677d84ed77d6a2ecf2070367adf82a95502e4bc557d5306972beeccba325f6i0",
    "57e1732448804c6d9a8cb9d1a51df66871d215d59388502cac8c8fb7a7eab2f7i0",
    "b240c4cbd51c1d450354f6f624f051c1bdfb95ab85afdedcd59e6105186b6ef9i0",
    "3de543d2613d3e3a33c17f332bce180ae6d4ca2060be9027896070f77f0c78fci0",
    "e17a4db28f45903ed41f0f9d485fdc87f83dae951decca2984119f6c304f1cfdi0",
    "2a8748526a68d7d37264e48c86bab8902ba11bfb0eb39ab8f640a33c2d6c4cfdi0",
    "8897a0f09a52caa688640ddf4295ebb2ec9dbb316529af17b3306d0965ea89fdi0",
    "77006382cd30d774a78b4660c6e37b5f7aa95cab6b05cf0f070004c034bc36fei0",
    "55f7318046c23ab95fcab1a43b7bbe1bbe44b013d592ed180d3eb5ada0d734ffi0",
    "c93bdf2a2ac25cc725db0b9673a629b4fcd4431bfe10ac87448e5b9ebfbafdffi0"];


    let base_url = "https://api.ordrare.org";
    let relative_url = "/inscription_json/";

    let client = reqwest::Client::builder()
      .timeout(std::time::Duration::from_secs(10))
      .build()?;

    let mut inscriptions = Vec::new();

    let mut cached_inscriptions: HashMap<&str, serde_json::Value> = HashMap::new();

    for ins_id in ins_ids.iter() {
        if let Some(cached_inscription) = cached_inscriptions.get(&ins_id[..]) {
            inscriptions.push(cached_inscription.clone());
        } else {
            let url = format!("{}{}{}", base_url, relative_url, ins_id);
            let inscription_response = client.get(&url).send().await?;
            let inscription_res_json = inscription_response.json::<serde_json::Value>().await?;
            cached_inscriptions.insert(&ins_id[..], inscription_res_json.clone());
            inscriptions.push(inscription_res_json);
        }
    }


    let result = serde_json::json!({
        "collection": {
            "collection_id": collection_id,
            "collection_author": "MahaDAO",
            "collection_description": "The People of Eden is first of its kind DeFi PFP collection combining art, robust utility, and storytelling to communicate the importance of financial freedom through an open yet decentralized ecosystem using $MAHA and $ARTH.",
            "collection_name": "People of Eden",
            "inscriptions": inscriptions,
        },
    });

    let data = serde_json::json!({"data":result});

    Ok(Json(data))
}


  async fn collection_json(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>
) -> ServerResult<Json<serde_json::Value>> {

    let collection = serde_json::json!({
      "collection_name": "People of Eden",
      "collection_author":"MahaDAO",
      "popularity_tag": "Exclusive collection",
      "popular_images": [
          "/content/7331f9bbed073f987dbf1967f0ce58970c50a4acb7cc5ac033db84ee1f76e600i0",
          "/content/79f14f5467c5ed92b28d44ed8c5469239cbe89ee88534b71e668f829303c5501i0",
          "/content/e77c2fbec4640430cab6e22756c92f84f5673bce17ab2b31b88912e00bcfc501i0",
          "/content/d341f3799e6ca24c419f9a18377a1d1583e6b3de7d3266e477f15957305be401i0"
      ],
      "volume": "abc",
      "floor_price": "xyz",
      "top_offer": "def"
    });

    let collections = [collection];
    let data = serde_json::json!({"collections":collections});

    Ok(Json(data))
  }


  async fn inscription(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(inscription_id): Path<InscriptionId>,
  ) -> ServerResult<PageHtml<InscriptionHtml>> {
    let entry = index
      .get_inscription_entry(inscription_id)?
      .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

    let inscription = index
      .get_inscription_by_id(inscription_id)?
      .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

    let satpoint = index
      .get_inscription_satpoint_by_id(inscription_id)?
      .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

    let output = index
      .get_transaction(satpoint.outpoint.txid)?
      .ok_or_not_found(|| format!("inscription {inscription_id} current transaction"))?
      .output
      .into_iter()
      .nth(satpoint.outpoint.vout.try_into().unwrap())
      .ok_or_not_found(|| format!("inscription {inscription_id} current transaction output"))?;

    let previous = if let Some(previous) = entry.number.checked_sub(1) {
      Some(
        index
          .get_inscription_id_by_inscription_number(previous)?
          .ok_or_not_found(|| format!("inscription {previous}"))?,
      )
    } else {
      None
    };

    let next = index.get_inscription_id_by_inscription_number(entry.number + 1)?;

    Ok(
      InscriptionHtml {
        chain: page_config.chain,
        genesis_fee: entry.fee,
        genesis_height: entry.height,
        inscription,
        inscription_id,
        next,
        number: entry.number,
        output,
        previous,
        sat: entry.sat,
        satpoint,
        timestamp: timestamp(entry.timestamp),
      }
      .page(page_config, index.has_sat_index()?),
    )
  }

  async fn inscriptions(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
  ) -> ServerResult<PageHtml<InscriptionsHtml>> {
    Self::inscriptions_inner(page_config, index, None).await
  }

  async fn inscriptions_from(
    Extension(page_config): Extension<Arc<PageConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(from): Path<u64>,
  ) -> ServerResult<PageHtml<InscriptionsHtml>> {
    Self::inscriptions_inner(page_config, index, Some(from)).await
  }

  async fn inscriptions_inner(
    page_config: Arc<PageConfig>,
    index: Arc<Index>,
    from: Option<u64>,
  ) -> ServerResult<PageHtml<InscriptionsHtml>> {
    let (inscriptions, prev, next) = index.get_latest_inscriptions_with_prev_and_next(100, from)?;
    Ok(
      InscriptionsHtml {
        inscriptions,
        next,
        prev,
      }
      .page(page_config, index.has_sat_index()?),
    )
  }

  async fn redirect_http_to_https(
    Extension(mut destination): Extension<String>,
    uri: Uri,
  ) -> Redirect {
    if let Some(path_and_query) = uri.path_and_query() {
      destination.push_str(path_and_query.as_str());
    }

    Redirect::to(&destination)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, reqwest::Url, std::net::TcpListener};

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
      Self::new_with_args(&[], &[])
    }

    fn new_with_sat_index() -> Self {
      Self::new_with_args(&["--index-sats"], &[])
    }

    fn new_with_args(ord_args: &[&str], server_args: &[&str]) -> Self {
      Self::new_server(test_bitcoincore_rpc::spawn(), None, ord_args, server_args)
    }

    fn new_with_bitcoin_rpc_server_and_config(
      bitcoin_rpc_server: test_bitcoincore_rpc::Handle,
      config: String,
    ) -> Self {
      Self::new_server(bitcoin_rpc_server, Some(config), &[], &[])
    }

    fn new_server(
      bitcoin_rpc_server: test_bitcoincore_rpc::Handle,
      config: Option<String>,
      ord_args: &[&str],
      server_args: &[&str],
    ) -> Self {
      let tempdir = TempDir::new().unwrap();

      let cookiefile = tempdir.path().join("cookie");

      fs::write(&cookiefile, "username:password").unwrap();

      let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

      let url = Url::parse(&format!("http://127.0.0.1:{port}")).unwrap();

      let config_args = match config {
        Some(config) => {
          let config_path = tempdir.path().join("ord.yaml");
          fs::write(&config_path, config).unwrap();
          format!("--config {}", config_path.display())
        }
        None => "".to_string(),
      };

      let (options, server) = parse_server_args(&format!(
        "ord --chain regtest --rpc-url {} --cookie-file {} --data-dir {} {config_args} {} server --http-port {} --address 127.0.0.1 {}",
        bitcoin_rpc_server.url(),
        cookiefile.to_str().unwrap(),
        tempdir.path().to_str().unwrap(),
        ord_args.join(" "),
        port,
        server_args.join(" "),
      ));

      let index = Arc::new(Index::open(&options).unwrap());
      let ord_server_handle = Handle::new();

      {
        let index = index.clone();
        let ord_server_handle = ord_server_handle.clone();
        thread::spawn(|| server.run(options, index, ord_server_handle).unwrap());
      }

      while index.statistic(crate::index::Statistic::Commits) == 0 {
        thread::sleep(Duration::from_millis(25));
      }

      let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

      for i in 0.. {
        match client.get(format!("http://127.0.0.1:{port}/status")).send() {
          Ok(_) => break,
          Err(err) => {
            if i == 400 {
              panic!("server failed to start: {err}");
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

    fn get(&self, path: impl AsRef<str>) -> reqwest::blocking::Response {
      if let Err(error) = self.index.update() {
        log::error!("{error}");
      }
      reqwest::blocking::get(self.join_url(path.as_ref())).unwrap()
    }

    fn join_url(&self, url: &str) -> Url {
      self.url.join(url).unwrap()
    }

    fn assert_response(&self, path: impl AsRef<str>, status: StatusCode, expected_response: &str) {
      let response = self.get(path);
      assert_eq!(response.status(), status, "{}", response.text().unwrap());
      pretty_assert_eq!(response.text().unwrap(), expected_response);
    }

    fn assert_response_regex(
      &self,
      path: impl AsRef<str>,
      status: StatusCode,
      regex: impl AsRef<str>,
    ) {
      let response = self.get(path);
      assert_eq!(response.status(), status);
      assert_regex_match!(response.text().unwrap(), regex.as_ref());
    }

    fn assert_response_csp(
      &self,
      path: impl AsRef<str>,
      status: StatusCode,
      content_security_policy: &str,
      regex: impl AsRef<str>,
    ) {
      let response = self.get(path);
      assert_eq!(response.status(), status);
      assert_eq!(
        response
          .headers()
          .get(header::CONTENT_SECURITY_POLICY,)
          .unwrap(),
        content_security_policy
      );
      assert_regex_match!(response.text().unwrap(), regex.as_ref());
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

    fn mine_blocks(&self, n: u64) -> Vec<bitcoin::Block> {
      let blocks = self.bitcoin_rpc_server.mine_blocks(n);
      self.index.update().unwrap();
      blocks
    }

    fn mine_blocks_with_subsidy(&self, n: u64, subsidy: u64) -> Vec<Block> {
      let blocks = self.bitcoin_rpc_server.mine_blocks_with_subsidy(n, subsidy);
      self.index.update().unwrap();
      blocks
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
        subcommand => panic!("unexpected subcommand: {subcommand:?}"),
      },
      Err(err) => panic!("error parsing arguments: {err}"),
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
    assert!(Arguments::try_parse_from([
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
    assert!(Arguments::try_parse_from([
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
    let arguments = Arguments::try_parse_from(["ord", "--data-dir", "foo", "server"]).unwrap();
    let acme_cache = Server::acme_cache(None, &arguments.options)
      .unwrap()
      .display()
      .to_string();
    assert!(
      acme_cache.contains(if cfg!(windows) {
        r"foo\acme-cache"
      } else {
        "foo/acme-cache"
      }),
      "{acme_cache}"
    )
  }

  #[test]
  fn acme_cache_flag_is_respected() {
    let arguments =
      Arguments::try_parse_from(["ord", "--data-dir", "foo", "server", "--acme-cache", "bar"])
        .unwrap();
    let acme_cache = Server::acme_cache(Some(&"bar".into()), &arguments.options)
      .unwrap()
      .display()
      .to_string();
    assert_eq!(acme_cache, "bar")
  }

  #[test]
  fn acme_domain_defaults_to_hostname() {
    let (_, server) = parse_server_args("ord server");
    assert_eq!(
      server.acme_domains().unwrap(),
      &[sys_info::hostname().unwrap()]
    );
  }

  #[test]
  fn acme_domain_flag_is_respected() {
    let (_, server) = parse_server_args("ord server --acme-domain example.com");
    assert_eq!(server.acme_domains().unwrap(), &["example.com"]);
  }

  #[test]
  fn install_sh_redirects_to_github() {
    TestServer::new().assert_redirect(
      "/install.sh",
      "https://raw.githubusercontent.com/casey/ord/master/install.sh",
    );
  }

  #[test]
  fn ordinal_redirects_to_sat() {
    TestServer::new().assert_redirect("/ordinal/0", "/sat/0");
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
  fn search_by_query_returns_sat() {
    TestServer::new().assert_redirect("/search?query=0", "/sat/0");
  }

  #[test]
  fn search_by_query_returns_inscription() {
    TestServer::new().assert_redirect(
      "/search?query=0000000000000000000000000000000000000000000000000000000000000000i0",
      "/inscription/0000000000000000000000000000000000000000000000000000000000000000i0",
    );
  }

  #[test]
  fn search_is_whitespace_insensitive() {
    TestServer::new().assert_redirect("/search/ 0 ", "/sat/0");
  }

  #[test]
  fn search_by_path_returns_sat() {
    TestServer::new().assert_redirect("/search/0", "/sat/0");
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
  fn search_for_inscription_id_returns_inscription() {
    TestServer::new().assert_redirect(
      "/search/0000000000000000000000000000000000000000000000000000000000000000i0",
      "/inscription/0000000000000000000000000000000000000000000000000000000000000000i0",
    );
  }

  #[test]
  fn http_to_https_redirect_with_path() {
    TestServer::new_with_args(&[], &["--redirect-http-to-https", "--https"]).assert_redirect(
      "/sat/0",
      &format!("https://{}/sat/0", sys_info::hostname().unwrap()),
    );
  }

  #[test]
  fn http_to_https_redirect_with_empty() {
    TestServer::new_with_args(&[], &["--redirect-http-to-https", "--https"])
      .assert_redirect("/", &format!("https://{}/", sys_info::hostname().unwrap()));
  }

  #[test]
  fn status() {
    TestServer::new().assert_response("/status", StatusCode::OK, "OK");
  }

  #[test]
  fn block_count_endpoint() {
    let test_server = TestServer::new();

    let response = test_server.get("/block-count");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "1");

    test_server.mine_blocks(1);

    let response = test_server.get("/block-count");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "2");
  }

  #[test]
  fn range_end_before_range_start_returns_400() {
    TestServer::new().assert_response(
      "/range/1/0",
      StatusCode::BAD_REQUEST,
      "range start greater than range end",
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
    TestServer::new().assert_response("/range/0/0", StatusCode::BAD_REQUEST, "empty range");
  }

  #[test]
  fn range() {
    TestServer::new().assert_response_regex(
      "/range/0/1",
      StatusCode::OK,
      r".*<title>Sat range 0–1</title>.*<h1>Sat range 0–1</h1>
<dl>
  <dt>value</dt><dd>1</dd>
  <dt>first</dt><dd><a href=/sat/0 class=mythic>0</a></dd>
</dl>.*",
    );
  }
  #[test]
  fn sat_number() {
    TestServer::new().assert_response_regex("/sat/0", StatusCode::OK, ".*<h1>Sat 0</h1>.*");
  }

  #[test]
  fn sat_decimal() {
    TestServer::new().assert_response_regex("/sat/0.0", StatusCode::OK, ".*<h1>Sat 0</h1>.*");
  }

  #[test]
  fn sat_degree() {
    TestServer::new().assert_response_regex("/sat/0°0′0″0‴", StatusCode::OK, ".*<h1>Sat 0</h1>.*");
  }

  #[test]
  fn sat_name() {
    TestServer::new().assert_response_regex(
      "/sat/nvtdijuwxlp",
      StatusCode::OK,
      ".*<h1>Sat 0</h1>.*",
    );
  }

  #[test]
  fn sat() {
    TestServer::new().assert_response_regex(
      "/sat/0",
      StatusCode::OK,
      ".*<title>Sat 0</title>.*<h1>Sat 0</h1>.*",
    );
  }

  #[test]
  fn block() {
    TestServer::new().assert_response_regex(
      "/block/0",
      StatusCode::OK,
      ".*<title>Block 0</title>.*<h1>Block 0</h1>.*",
    );
  }

  #[test]
  fn sat_out_of_range() {
    TestServer::new().assert_response(
      "/sat/2099999997690000",
      StatusCode::BAD_REQUEST,
      "Invalid URL: invalid sat",
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
  fn output_with_sat_index() {
    let txid = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b";
    TestServer::new_with_sat_index().assert_response_regex(
      format!("/output/{txid}:0"),
      StatusCode::OK,
      format!(
        ".*<title>Output {txid}:0</title>.*<h1>Output <span class=monospace>{txid}:0</span></h1>
<dl>
  <dt>value</dt><dd>5000000000</dd>
  <dt>script pubkey</dt><dd class=monospace>OP_PUSHBYTES_65 [[:xdigit:]]{{130}} OP_CHECKSIG</dd>
  <dt>transaction</dt><dd><a class=monospace href=/tx/{txid}>{txid}</a></dd>
</dl>
<h2>1 Sat Range</h2>
<ul class=monospace>
  <li><a href=/range/0/5000000000 class=mythic>0–5000000000</a></li>
</ul>.*"
      ),
    );
  }

  #[test]
  fn output_without_sat_index() {
    let txid = "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b";
    TestServer::new().assert_response_regex(
      format!("/output/{txid}:0"),
      StatusCode::OK,
      format!(
        ".*<title>Output {txid}:0</title>.*<h1>Output <span class=monospace>{txid}:0</span></h1>
<dl>
  <dt>value</dt><dd>5000000000</dd>
  <dt>script pubkey</dt><dd class=monospace>OP_PUSHBYTES_65 [[:xdigit:]]{{130}} OP_CHECKSIG</dd>
  <dt>transaction</dt><dd><a class=monospace href=/tx/{txid}>{txid}</a></dd>
</dl>.*"
      ),
    );
  }

  #[test]
  fn null_output_is_initially_empty() {
    let txid = "0000000000000000000000000000000000000000000000000000000000000000";
    TestServer::new_with_sat_index().assert_response_regex(
      format!("/output/{txid}:4294967295"),
      StatusCode::OK,
      format!(
        ".*<title>Output {txid}:4294967295</title>.*<h1>Output <span class=monospace>{txid}:4294967295</span></h1>
<dl>
  <dt>value</dt><dd>0</dd>
  <dt>script pubkey</dt><dd class=monospace></dd>
  <dt>transaction</dt><dd><a class=monospace href=/tx/{txid}>{txid}</a></dd>
</dl>
<h2>0 Sat Ranges</h2>
<ul class=monospace>
</ul>.*"
      ),
    );
  }

  #[test]
  fn null_output_receives_lost_sats() {
    let server = TestServer::new_with_sat_index();

    server.mine_blocks_with_subsidy(1, 0);

    let txid = "0000000000000000000000000000000000000000000000000000000000000000";

    server.assert_response_regex(
      format!("/output/{txid}:4294967295"),
      StatusCode::OK,
      format!(
        ".*<title>Output {txid}:4294967295</title>.*<h1>Output <span class=monospace>{txid}:4294967295</span></h1>
<dl>
  <dt>value</dt><dd>5000000000</dd>
  <dt>script pubkey</dt><dd class=monospace></dd>
  <dt>transaction</dt><dd><a class=monospace href=/tx/{txid}>{txid}</a></dd>
</dl>
<h2>1 Sat Range</h2>
<ul class=monospace>
  <li><a href=/range/5000000000/10000000000 class=uncommon>5000000000–10000000000</a></li>
</ul>.*"
      ),
    );
  }

  #[test]
  fn unknown_output_returns_404() {
    TestServer::new().assert_response(
      "/output/0000000000000000000000000000000000000000000000000000000000000000:0",
      StatusCode::NOT_FOUND,
      "output 0000000000000000000000000000000000000000000000000000000000000000:0 not found",
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

    test_server.mine_blocks(1);

    test_server.assert_response_regex(
    "/",
    StatusCode::OK,
    ".*<title>Ordinals</title>.*
<h2>Latest Blocks</h2>
<ol start=1 reversed class=blocks>
  <li><a href=/block/[[:xdigit:]]{64}>[[:xdigit:]]{64}</a></li>
  <li><a href=/block/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f>000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f</a></li>
</ol>.*",
  );
  }

  #[test]
  fn nav_displays_chain() {
    TestServer::new().assert_response_regex(
      "/",
      StatusCode::OK,
      ".*<a href=/>Ordinals<sup>regtest</sup></a>.*",
    );
  }

  #[test]
  fn home_block_limit() {
    let test_server = TestServer::new();

    test_server.mine_blocks(101);

    test_server.assert_response_regex(
    "/",
    StatusCode::OK,
    ".*<ol start=101 reversed class=blocks>\n(  <li><a href=/block/[[:xdigit:]]{64}>[[:xdigit:]]{64}</a></li>\n){100}</ol>.*"
  );
  }

  #[test]
  fn block_not_found() {
    TestServer::new().assert_response(
      "/block/467a86f0642b1d284376d13a98ef58310caa49502b0f9a560ee222e0a122fe16",
      StatusCode::NOT_FOUND,
      "block 467a86f0642b1d284376d13a98ef58310caa49502b0f9a560ee222e0a122fe16 not found",
    );
  }

  #[test]
  fn unmined_sat() {
    TestServer::new().assert_response_regex(
      "/sat/0",
      StatusCode::OK,
      ".*<dt>timestamp</dt><dd><time>2009-01-03 18:15:05 UTC</time></dd>.*",
    );
  }

  #[test]
  fn mined_sat() {
    TestServer::new().assert_response_regex(
      "/sat/5000000000",
      StatusCode::OK,
      ".*<dt>timestamp</dt><dd><time>.*</time> \\(expected\\)</dd>.*",
    );
  }

  #[test]
  fn static_asset() {
    TestServer::new().assert_response_regex(
      "/static/index.css",
      StatusCode::OK,
      r".*\.rare \{
  background-color: var\(--rare\);
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
    test_server.mine_blocks(1);
    test_server.assert_response_regex("/clock", StatusCode::OK, ".*<text.*>1</text>.*");
  }

  #[test]
  fn block_by_hash() {
    let test_server = TestServer::new();

    test_server.mine_blocks(1);
    let transaction = TransactionTemplate {
      inputs: &[(1, 0, 0)],
      fee: 0,
      ..Default::default()
    };
    test_server.bitcoin_rpc_server.broadcast_tx(transaction);
    let block_hash = test_server.mine_blocks(1)[0].block_hash();

    test_server.assert_response_regex(
      format!("/block/{block_hash}"),
      StatusCode::OK,
      ".*<h1>Block 2</h1>.*",
    );
  }

  #[test]
  fn block_by_height() {
    let test_server = TestServer::new();

    test_server.assert_response_regex("/block/0", StatusCode::OK, ".*<h1>Block 0</h1>.*");
  }

  #[test]
  fn transaction() {
    let test_server = TestServer::new();

    let coinbase_tx = test_server.mine_blocks(1)[0].txdata[0].clone();
    let txid = coinbase_tx.txid();

    test_server.assert_response_regex(
      format!("/tx/{txid}"),
      StatusCode::OK,
      format!(
        ".*<title>Transaction {txid}</title>.*<h1>Transaction <span class=monospace>{txid}</span></h1>
<h2>1 Input</h2>
<ul>
  <li><a class=monospace href=/output/0000000000000000000000000000000000000000000000000000000000000000:4294967295>0000000000000000000000000000000000000000000000000000000000000000:4294967295</a></li>
</ul>
<h2>1 Output</h2>
<ul class=monospace>
  <li>
    <a href=/output/30f2f037629c6a21c1f40ed39b9bd6278df39762d68d07f49582b23bcb23386a:0 class=monospace>
      30f2f037629c6a21c1f40ed39b9bd6278df39762d68d07f49582b23bcb23386a:0
    </a>
    <dl>
      <dt>value</dt><dd>5000000000</dd>
      <dt>script pubkey</dt><dd class=monospace></dd>
    </dl>
  </li>
</ul>.*"
      ),
    );
  }

  #[test]
  fn detect_reorg() {
    let test_server = TestServer::new();

    test_server.mine_blocks(1);

    test_server.assert_response("/status", StatusCode::OK, "OK");

    test_server.bitcoin_rpc_server.invalidate_tip();
    test_server.bitcoin_rpc_server.mine_blocks(2);

    test_server.assert_response_regex("/status", StatusCode::OK, "reorg detected.*");
  }

  #[test]
  fn rare_with_index() {
    TestServer::new_with_sat_index().assert_response(
      "/rare.txt",
      StatusCode::OK,
      "sat\tsatpoint
0\t4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0
",
    );
  }

  #[test]
  fn rare_without_sat_index() {
    TestServer::new().assert_response(
      "/rare.txt",
      StatusCode::NOT_FOUND,
      "tracking rare sats requires index created with `--index-sats` flag",
    );
  }

  #[test]
  fn show_rare_txt_in_header_with_sat_index() {
    TestServer::new_with_sat_index().assert_response_regex(
      "/",
      StatusCode::OK,
      ".*
      <a href=/clock>Clock</a>
      <a href=/rare.txt>rare.txt</a>
      <form action=/search method=get>.*",
    );
  }

  #[test]
  fn rare_sat_location() {
    TestServer::new_with_sat_index().assert_response_regex(
      "/sat/0",
      StatusCode::OK,
      ".*>4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0:0<.*",
    );
  }

  #[test]
  fn dont_show_rare_txt_in_header_without_sat_index() {
    TestServer::new().assert_response_regex(
      "/",
      StatusCode::OK,
      ".*
      <a href=/clock>Clock</a>
      <form action=/search method=get>.*",
    );
  }

  #[test]
  fn input() {
    TestServer::new().assert_response_regex(
      "/input/0/0/0",
      StatusCode::OK,
      ".*<title>Input /0/0/0</title>.*<h1>Input /0/0/0</h1>.*<dt>text</dt><dd>.*The Times 03/Jan/2009 Chancellor on brink of second bailout for banks</dd>.*",
    );
  }

  #[test]
  fn input_missing() {
    TestServer::new().assert_response(
      "/input/1/1/1",
      StatusCode::NOT_FOUND,
      "input /1/1/1 not found",
    );
  }

  #[test]
  fn commits_are_tracked() {
    let server = TestServer::new();

    assert_eq!(server.index.statistic(crate::index::Statistic::Commits), 1);

    let info = server.index.info().unwrap();
    assert_eq!(info.transactions.len(), 1);
    assert_eq!(info.transactions[0].starting_block_count, 0);

    server.index.update().unwrap();

    assert_eq!(server.index.statistic(crate::index::Statistic::Commits), 1);

    let info = server.index.info().unwrap();
    assert_eq!(info.transactions.len(), 1);
    assert_eq!(info.transactions[0].starting_block_count, 0);

    server.mine_blocks(1);

    thread::sleep(Duration::from_millis(10));
    server.index.update().unwrap();

    assert_eq!(server.index.statistic(crate::index::Statistic::Commits), 2);

    let info = server.index.info().unwrap();
    assert_eq!(info.transactions.len(), 2);
    assert_eq!(info.transactions[0].starting_block_count, 0);
    assert_eq!(info.transactions[1].starting_block_count, 1);
    assert!(
      info.transactions[1].starting_timestamp - info.transactions[0].starting_timestamp >= 10
    );
  }

  #[test]
  fn outputs_traversed_are_tracked() {
    let server = TestServer::new_with_sat_index();

    assert_eq!(
      server
        .index
        .statistic(crate::index::Statistic::OutputsTraversed),
      1
    );

    server.index.update().unwrap();

    assert_eq!(
      server
        .index
        .statistic(crate::index::Statistic::OutputsTraversed),
      1
    );

    server.mine_blocks(2);

    server.index.update().unwrap();

    assert_eq!(
      server
        .index
        .statistic(crate::index::Statistic::OutputsTraversed),
      3
    );
  }

  #[test]
  fn coinbase_sat_ranges_are_tracked() {
    let server = TestServer::new_with_sat_index();

    assert_eq!(
      server.index.statistic(crate::index::Statistic::SatRanges),
      1
    );

    server.mine_blocks(1);

    assert_eq!(
      server.index.statistic(crate::index::Statistic::SatRanges),
      2
    );

    server.mine_blocks(1);

    assert_eq!(
      server.index.statistic(crate::index::Statistic::SatRanges),
      3
    );
  }

  #[test]
  fn split_sat_ranges_are_tracked() {
    let server = TestServer::new_with_sat_index();

    assert_eq!(
      server.index.statistic(crate::index::Statistic::SatRanges),
      1
    );

    server.mine_blocks(1);
    server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      outputs: 2,
      fee: 0,
      ..Default::default()
    });
    server.mine_blocks(1);

    assert_eq!(
      server.index.statistic(crate::index::Statistic::SatRanges),
      4,
    );
  }

  #[test]
  fn fee_sat_ranges_are_tracked() {
    let server = TestServer::new_with_sat_index();

    assert_eq!(
      server.index.statistic(crate::index::Statistic::SatRanges),
      1
    );

    server.mine_blocks(1);
    server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      outputs: 2,
      fee: 2,
      ..Default::default()
    });
    server.mine_blocks(1);

    assert_eq!(
      server.index.statistic(crate::index::Statistic::SatRanges),
      5,
    );
  }

  #[test]
  fn content_response_no_content() {
    assert_eq!(
      Server::content_response(Inscription::new(
        Some("text/plain".as_bytes().to_vec()),
        None
      )),
      None
    );
  }

  #[test]
  fn content_response_with_content() {
    let (headers, body) = Server::content_response(Inscription::new(
      Some("text/plain".as_bytes().to_vec()),
      Some(vec![1, 2, 3]),
    ))
    .unwrap();

    assert_eq!(headers["content-type"], "text/plain");
    assert_eq!(body, vec![1, 2, 3]);
  }

  #[test]
  fn content_response_no_content_type() {
    let (headers, body) =
      Server::content_response(Inscription::new(None, Some(Vec::new()))).unwrap();

    assert_eq!(headers["content-type"], "application/octet-stream");
    assert!(body.is_empty());
  }

  #[test]
  fn text_preview() {
    let server = TestServer::new();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("text/plain;charset=utf-8", "hello").to_witness(),
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_csp(
      format!("/preview/{}", InscriptionId::from(txid)),
      StatusCode::OK,
      "default-src 'self'",
      ".*<pre>hello</pre>.*",
    );
  }

  #[test]
  fn text_preview_returns_error_when_content_is_not_utf8() {
    let server = TestServer::new();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("text/plain;charset=utf-8", b"\xc3\x28").to_witness(),
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response(
      format!("/preview/{}", InscriptionId::from(txid)),
      StatusCode::INTERNAL_SERVER_ERROR,
      "Internal Server Error",
    );
  }

  #[test]
  fn text_preview_text_is_escaped() {
    let server = TestServer::new();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription(
        "text/plain;charset=utf-8",
        "<script>alert('hello');</script>",
      )
      .to_witness(),
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_csp(
      format!("/preview/{}", InscriptionId::from(txid)),
      StatusCode::OK,
      "default-src 'self'",
      r".*<pre>&lt;script&gt;alert\(&apos;hello&apos;\);&lt;/script&gt;</pre>.*",
    );
  }

  #[test]
  fn audio_preview() {
    let server = TestServer::new();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("audio/flac", "hello").to_witness(),
      ..Default::default()
    });
    let inscription_id = InscriptionId::from(txid);

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/preview/{inscription_id}"),
      StatusCode::OK,
      format!(r".*<audio .*>\s*<source src=/content/{inscription_id}>.*"),
    );
  }

  #[test]
  fn pdf_preview() {
    let server = TestServer::new();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("application/pdf", "hello").to_witness(),
      ..Default::default()
    });
    let inscription_id = InscriptionId::from(txid);

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/preview/{inscription_id}"),
      StatusCode::OK,
      format!(r".*<canvas data-inscription={inscription_id}></canvas>.*"),
    );
  }

  #[test]
  fn image_preview() {
    let server = TestServer::new();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("image/png", "hello").to_witness(),
      ..Default::default()
    });
    let inscription_id = InscriptionId::from(txid);

    server.mine_blocks(1);

    server.assert_response_csp(
      format!("/preview/{inscription_id}"),
      StatusCode::OK,
      "default-src 'self' 'unsafe-inline'",
      format!(r".*background-image: url\(/content/{inscription_id}\);.*"),
    );
  }

  #[test]
  fn iframe_preview() {
    let server = TestServer::new();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("text/html;charset=utf-8", "hello").to_witness(),
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_csp(
      format!("/preview/{}", InscriptionId::from(txid)),
      StatusCode::OK,
      "default-src 'unsafe-eval' 'unsafe-inline' data:",
      "hello",
    );
  }

  #[test]
  fn unknown_preview() {
    let server = TestServer::new();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("text/foo", "hello").to_witness(),
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_csp(
      format!("/preview/{}", InscriptionId::from(txid)),
      StatusCode::OK,
      "default-src 'self'",
      fs::read_to_string("templates/preview-unknown.html").unwrap(),
    );
  }

  #[test]
  fn video_preview() {
    let server = TestServer::new();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("video/webm", "hello").to_witness(),
      ..Default::default()
    });
    let inscription_id = InscriptionId::from(txid);

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/preview/{inscription_id}"),
      StatusCode::OK,
      format!(r".*<video .*>\s*<source src=/content/{inscription_id}>.*"),
    );
  }

  #[test]
  fn inscription_page_title() {
    let server = TestServer::new_with_sat_index();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("text/foo", "hello").to_witness(),
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{}", InscriptionId::from(txid)),
      StatusCode::OK,
      ".*<title>Inscription 0</title>.*",
    );
  }

  #[test]
  fn inscription_page_has_sat_when_sats_are_tracked() {
    let server = TestServer::new_with_sat_index();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("text/foo", "hello").to_witness(),
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{}", InscriptionId::from(txid)),
      StatusCode::OK,
      r".*<dt>sat</dt>\s*<dd><a href=/sat/5000000000>5000000000</a></dd>\s*<dt>preview</dt>.*",
    );
  }

  #[test]
  fn inscription_page_does_not_have_sat_when_sats_are_not_tracked() {
    let server = TestServer::new();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("text/foo", "hello").to_witness(),
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{}", InscriptionId::from(txid)),
      StatusCode::OK,
      r".*<dt>output value</dt>\s*<dd>5000000000</dd>\s*<dt>preview</dt>.*",
    );
  }

  #[test]
  fn strict_transport_security_header_is_set() {
    assert_eq!(
      TestServer::new()
        .get("/status")
        .headers()
        .get(header::STRICT_TRANSPORT_SECURITY)
        .unwrap(),
      "max-age=31536000; includeSubDomains; preload",
    );
  }

  #[test]
  fn feed() {
    let server = TestServer::new_with_sat_index();
    server.mine_blocks(1);

    server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("text/foo", "hello").to_witness(),
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_regex(
      "/feed.xml",
      StatusCode::OK,
      ".*<title>Inscription 0</title>.*",
    );
  }

  #[test]
  fn inscription_with_unknown_type_and_no_body_has_unknown_preview() {
    let server = TestServer::new_with_sat_index();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: Inscription::new(Some("foo/bar".as_bytes().to_vec()), None).to_witness(),
      ..Default::default()
    });

    let inscription_id = InscriptionId::from(txid);

    server.mine_blocks(1);

    server.assert_response(
      format!("/preview/{inscription_id}"),
      StatusCode::OK,
      &fs::read_to_string("templates/preview-unknown.html").unwrap(),
    );
  }

  #[test]
  fn inscription_with_known_type_and_no_body_has_unknown_preview() {
    let server = TestServer::new_with_sat_index();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: Inscription::new(Some("image/png".as_bytes().to_vec()), None).to_witness(),
      ..Default::default()
    });

    let inscription_id = InscriptionId::from(txid);

    server.mine_blocks(1);

    server.assert_response(
      format!("/preview/{inscription_id}"),
      StatusCode::OK,
      &fs::read_to_string("templates/preview-unknown.html").unwrap(),
    );
  }

  #[test]
  fn content_responses_have_cache_control_headers() {
    let server = TestServer::new();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("text/foo", "hello").to_witness(),
      ..Default::default()
    });

    server.mine_blocks(1);

    let response = server.get(format!("/content/{}", InscriptionId::from(txid)));

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
      response.headers().get(header::CACHE_CONTROL).unwrap(),
      "max-age=31536000, immutable"
    );
  }

  #[test]
  fn inscriptions_page_with_no_prev_or_next() {
    TestServer::new_with_sat_index().assert_response_regex(
      "/inscriptions",
      StatusCode::OK,
      ".*prev\nnext.*",
    );
  }

  #[test]
  fn inscriptions_page_with_no_next() {
    let server = TestServer::new_with_sat_index();

    for i in 0..101 {
      server.mine_blocks(1);
      server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(i + 1, 0, 0)],
        witness: inscription("text/foo", "hello").to_witness(),
        ..Default::default()
      });
    }

    server.mine_blocks(1);

    server.assert_response_regex(
      "/inscriptions",
      StatusCode::OK,
      ".*<a class=prev href=/inscriptions/0>prev</a>\nnext.*",
    );
  }

  #[test]
  fn inscriptions_page_with_no_prev() {
    let server = TestServer::new_with_sat_index();

    for i in 0..101 {
      server.mine_blocks(1);
      server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(i + 1, 0, 0)],
        witness: inscription("text/foo", "hello").to_witness(),
        ..Default::default()
      });
    }

    server.mine_blocks(1);

    server.assert_response_regex(
      "/inscriptions/0",
      StatusCode::OK,
      ".*prev\n<a class=next href=/inscriptions/100>next</a>.*",
    );
  }

  #[test]
  fn resonses_are_gzipped() {
    let server = TestServer::new();

    let mut headers = HeaderMap::new();

    headers.insert(header::ACCEPT_ENCODING, "gzip".parse().unwrap());

    let response = reqwest::blocking::Client::builder()
      .default_headers(headers)
      .build()
      .unwrap()
      .get(server.join_url("/"))
      .send()
      .unwrap();

    assert_eq!(
      response.headers().get(header::CONTENT_ENCODING).unwrap(),
      "gzip"
    );
  }

  #[test]
  fn resonses_are_brotlied() {
    let server = TestServer::new();

    let mut headers = HeaderMap::new();

    headers.insert(header::ACCEPT_ENCODING, "br".parse().unwrap());

    let response = reqwest::blocking::Client::builder()
      .default_headers(headers)
      .build()
      .unwrap()
      .get(server.join_url("/"))
      .send()
      .unwrap();

    assert_eq!(
      response.headers().get(header::CONTENT_ENCODING).unwrap(),
      "br"
    );
  }

  #[test]
  fn inscriptions_can_be_hidden_with_config() {
    let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
    bitcoin_rpc_server.mine_blocks(1);
    let txid = bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("text/plain;charset=utf-8", "hello").to_witness(),
      ..Default::default()
    });
    let inscription = InscriptionId::from(txid);
    bitcoin_rpc_server.mine_blocks(1);

    let server = TestServer::new_with_bitcoin_rpc_server_and_config(
      bitcoin_rpc_server,
      format!("\"hidden\":\n - {inscription}"),
    );

    server.assert_response(
      format!("/preview/{inscription}"),
      StatusCode::OK,
      &fs::read_to_string("templates/preview-unknown.html").unwrap(),
    );

    server.assert_response(
      format!("/content/{inscription}"),
      StatusCode::OK,
      &fs::read_to_string("templates/preview-unknown.html").unwrap(),
    );
  }
}
