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
      let update_thread = thread::spawn(move || loop {
        if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
          break;
        }
        if let Err(error) = clone.update() {
          log::warn!("{error}");
        }
        thread::sleep(Duration::from_millis(5000));
      });
      UPDATE_THREAD.lock().unwrap().replace(update_thread);

      let config = options.load_config()?;
      let acme_domains = self.acme_domains()?;

      let page_config = Arc::new(PageConfig {
        chain: options.chain(),
        domain: acme_domains.first().cloned(),
      });

      let router = Router::new()
        .route("/", get(Self::home))
        .route("/block/:query", get(Self::block))
        .route("/blockcount", get(Self::block_count))
        .route("/blockheight", get(Self::block_height))
        .route("/blockhash", get(Self::block_hash))
        .route("/blockhash/:height", get(Self::block_hash_from_height))
        .route("/blocktime", get(Self::block_time))
        .route("/bounties", get(Self::bounties))
        .route("/clock", get(Self::clock))
        .route("/content/:inscription_id", get(Self::content))
        .route("/faq", get(Self::faq))
        .route("/favicon.ico", get(Self::favicon))
        .route("/feed.xml", get(Self::feed))
        .route("/input/:block/:transaction/:input", get(Self::input))
        .route("/inscription/:inscription_id", get(Self::inscription))
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
      Ok(vec![System::new()
        .host_name()
        .ok_or(anyhow!("no hostname found"))?])
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
    index.block_height()?.ok_or_not_found(|| "genesis block")
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
        blocktime: index.block_time(sat.height())?,
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

    let output = if outpoint == OutPoint::null() || outpoint == unbound_outpoint() {
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

  async fn block_height(Extension(index): Extension<Arc<Index>>) -> ServerResult<String> {
    Ok(
      index
        .block_height()?
        .ok_or_not_found(|| "blockheight")?
        .to_string(),
    )
  }

  async fn block_hash(Extension(index): Extension<Arc<Index>>) -> ServerResult<String> {
    Ok(
      index
        .block_hash(None)?
        .ok_or_not_found(|| "blockhash")?
        .to_string(),
    )
  }

  async fn block_hash_from_height(
    Extension(index): Extension<Arc<Index>>,
    Path(height): Path<u64>,
  ) -> ServerResult<String> {
    Ok(
      index
        .block_hash(Some(height))?
        .ok_or_not_found(|| "blockhash")?
        .to_string(),
    )
  }

  async fn block_time(Extension(index): Extension<Arc<Index>>) -> ServerResult<String> {
    Ok(
      index
        .block_time(index.block_height()?.ok_or_not_found(|| "blocktime")?)?
        .unix_timestamp()
        .to_string(),
    )
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
      HeaderValue::from_static("default-src 'self' 'unsafe-eval' 'unsafe-inline' data: blob:"),
    );
    headers.append(
      header::CONTENT_SECURITY_POLICY,
      HeaderValue::from_static("default-src *:*/content/ *:*/blockheight *:*/blockhash *:*/blockhash/ *:*/blocktime 'unsafe-eval' 'unsafe-inline' data: blob:"),
    );

    let body = inscription.into_body();
    let cache_control = match body {
      Some(_) => "max-age=31536000, immutable",
      None => "max-age=600",
    };
    headers.insert(
      header::CACHE_CONTROL,
      HeaderValue::from_str(cache_control).unwrap(),
    );

    Some((headers, body?))
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

    match inscription.media() {
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
    }
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

    let output = if satpoint.outpoint == unbound_outpoint() {
      None
    } else {
      Some(
        index
          .get_transaction(satpoint.outpoint.txid)?
          .ok_or_not_found(|| format!("inscription {inscription_id} current transaction"))?
          .output
          .into_iter()
          .nth(satpoint.outpoint.vout.try_into().unwrap())
          .ok_or_not_found(|| format!("inscription {inscription_id} current transaction output"))?,
      )
    };

    let previous = index.get_inscription_id_by_inscription_number(entry.number - 1)?;

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
    Path(from): Path<i64>,
  ) -> ServerResult<PageHtml<InscriptionsHtml>> {
    Self::inscriptions_inner(page_config, index, Some(from)).await
  }

  async fn inscriptions_inner(
    page_config: Arc<PageConfig>,
    index: Arc<Index>,
    from: Option<i64>,
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

    fn new_with_regtest() -> Self {
      Self::new_server(
        test_bitcoincore_rpc::builder()
          .network(bitcoin::Network::Regtest)
          .build(),
        None,
        &["--chain", "regtest"],
        &[],
      )
    }

    fn new_with_regtest_with_index_sats() -> Self {
      Self::new_server(
        test_bitcoincore_rpc::builder()
          .network(bitcoin::Network::Regtest)
          .build(),
        None,
        &["--chain", "regtest", "--index-sats"],
        &[],
      )
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
        "ord --rpc-url {} --cookie-file {} --data-dir {} {config_args} {} server --http-port {} --address 127.0.0.1 {}",
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
      &[System::new().host_name().unwrap()]
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
      &format!("https://{}/sat/0", System::new().host_name().unwrap()),
    );
  }

  #[test]
  fn http_to_https_redirect_with_empty() {
    TestServer::new_with_args(&[], &["--redirect-http-to-https", "--https"]).assert_redirect(
      "/",
      &format!("https://{}/", System::new().host_name().unwrap()),
    );
  }

  #[test]
  fn status() {
    TestServer::new().assert_response("/status", StatusCode::OK, "OK");
  }

  #[test]
  fn block_count_endpoint() {
    let test_server = TestServer::new();

    let response = test_server.get("/blockcount");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "1");

    test_server.mine_blocks(1);

    let response = test_server.get("/blockcount");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "2");
  }

  #[test]
  fn block_height_endpoint() {
    let test_server = TestServer::new();

    let response = test_server.get("/blockheight");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "0");

    test_server.mine_blocks(2);

    let response = test_server.get("/blockheight");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "2");
  }

  #[test]
  fn block_hash_endpoint() {
    let test_server = TestServer::new();

    let response = test_server.get("/blockhash");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
      response.text().unwrap(),
      "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
    );
  }

  #[test]
  fn block_hash_from_height_endpoint() {
    let test_server = TestServer::new();

    let response = test_server.get("/blockhash/0");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
      response.text().unwrap(),
      "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
    );
  }

  #[test]
  fn block_time_endpoint() {
    let test_server = TestServer::new();

    let response = test_server.get("/blocktime");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().unwrap(), "1231006505");
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
  fn unbound_output_recieves_unbound_inscriptions() {
    let server = TestServer::new_with_regtest();

    server.mine_blocks(1);

    server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      fee: 50 * 100_000_000,
      ..Default::default()
    });

    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0)],
      witness: inscription("text/plain;charset=utf-8", "hello").to_witness(),
      ..Default::default()
    });

    server.mine_blocks(1);

    let inscription_id = InscriptionId::from(txid);

    server.assert_response_regex(
      format!("/inscription/{}", inscription_id),
      StatusCode::OK,
      format!(
        ".*<dl>
  <dt>id</dt>
  <dd class=monospace>{inscription_id}</dd>
  <dt>preview</dt>.*<dt>output</dt>
  <dd><a class=monospace href=/output/0000000000000000000000000000000000000000000000000000000000000000:0>0000000000000000000000000000000000000000000000000000000000000000:0 \\(unbound\\)</a></dd>.*"
      ),
    );
  }

  #[test]
  fn unbound_output_returns_200() {
    TestServer::new().assert_response_regex(
      "/output/0000000000000000000000000000000000000000000000000000000000000000:0",
      StatusCode::OK,
      ".*",
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
    TestServer::new_with_regtest().assert_response_regex(
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
    let server = TestServer::new_with_regtest();
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
    let server = TestServer::new_with_regtest();
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
    let server = TestServer::new_with_regtest();
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
    let server = TestServer::new_with_regtest();
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
    let server = TestServer::new_with_regtest();
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
    let server = TestServer::new_with_regtest();
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
    let server = TestServer::new_with_regtest();
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
      "default-src 'self' 'unsafe-eval' 'unsafe-inline' data: blob:",
      "hello",
    );
  }

  #[test]
  fn unknown_preview() {
    let server = TestServer::new_with_regtest();
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
    let server = TestServer::new_with_regtest();
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
    let server = TestServer::new_with_regtest_with_index_sats();
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
    let server = TestServer::new_with_regtest_with_index_sats();
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
    let server = TestServer::new_with_regtest();
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
    let server = TestServer::new_with_regtest_with_index_sats();
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
    let server = TestServer::new_with_regtest_with_index_sats();
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
    let server = TestServer::new_with_regtest_with_index_sats();
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
    let server = TestServer::new_with_regtest();
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
    TestServer::new_with_regtest_with_index_sats().assert_response_regex(
      "/inscriptions",
      StatusCode::OK,
      ".*prev\nnext.*",
    );
  }

  #[test]
  fn inscriptions_page_with_no_next() {
    let server = TestServer::new_with_regtest_with_index_sats();

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
    let server = TestServer::new_with_regtest_with_index_sats();

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
  fn responses_are_gzipped() {
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
  fn responses_are_brotlied() {
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
