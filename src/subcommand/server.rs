use {
  self::{
    accept_encoding::AcceptEncoding,
    accept_json::AcceptJson,
    deserialize_from_str::DeserializeFromStr,
    error::{OptionExt, ServerError, ServerResult},
  },
  super::*,
  crate::{
    ordzaar::inscriptions::{InscriptionData, InscriptionIds},
    ordzaar::ordinals::get_ordinals,
    server_config::ServerConfig,
    templates::{
      BlockHtml, BlockJson, BlocksHtml, ChildrenHtml, ChildrenJson, ClockSvg, CollectionsHtml,
      HomeHtml, InputHtml, InscriptionHtml, InscriptionJson, InscriptionsBlockHtml,
      InscriptionsHtml, InscriptionsJson, OutputHtml, OutputJson, PageContent, PageHtml,
      PreviewAudioHtml, PreviewCodeHtml, PreviewFontHtml, PreviewImageHtml, PreviewMarkdownHtml,
      PreviewModelHtml, PreviewPdfHtml, PreviewTextHtml, PreviewUnknownHtml, PreviewVideoHtml,
      RangeHtml, RareTxt, RuneHtml, RuneJson, RunesHtml, RunesJson, SatHtml, SatInscriptionJson,
      SatInscriptionsJson, SatJson, TransactionHtml,
    },
  },
  axum::{
    body,
    extract::{Extension, Json, Path, Query},
    headers::UserAgent,
    http::{header, HeaderMap, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router, TypedHeader,
  },
  axum_server::Handle,
  brotli::Decompressor,
  rust_embed::RustEmbed,
  rustls_acme::{
    acme::{LETS_ENCRYPT_PRODUCTION_DIRECTORY, LETS_ENCRYPT_STAGING_DIRECTORY},
    axum::AxumAcceptor,
    caches::DirCache,
    AcmeConfig,
  },
  std::{cmp::Ordering, io::Read, str, sync::Arc},
  tokio_stream::StreamExt,
  tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
  },
};

mod accept_encoding;
mod accept_json;
mod error;

#[derive(Copy, Clone)]
pub(crate) enum InscriptionQuery {
  Id(InscriptionId),
  Number(i32),
}

impl FromStr for InscriptionQuery {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(if s.contains('i') {
      Self::Id(s.parse()?)
    } else {
      Self::Number(s.parse()?)
    })
  }
}

impl Display for InscriptionQuery {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Id(id) => write!(f, "{id}"),
      Self::Number(number) => write!(f, "{number}"),
    }
  }
}

enum BlockQuery {
  Height(u32),
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
  #[arg(
    long,
    help = "Listen on <ADDRESS> for incoming requests. [default: 0.0.0.0]"
  )]
  address: Option<String>,
  #[arg(
    long,
    help = "Request ACME TLS certificate for <ACME_DOMAIN>. This ord instance must be reachable at <ACME_DOMAIN>:443 to respond to Let's Encrypt ACME challenges."
  )]
  acme_domain: Vec<String>,
  #[arg(
    long,
    help = "Use <CSP_ORIGIN> in Content-Security-Policy header. Set this to the public-facing URL of your ord instance."
  )]
  csp_origin: Option<String>,
  #[arg(
    long,
    help = "Listen on <HTTP_PORT> for incoming HTTP requests. [default: 80]"
  )]
  http_port: Option<u16>,
  #[arg(
    long,
    group = "port",
    help = "Listen on <HTTPS_PORT> for incoming HTTPS requests. [default: 443]"
  )]
  https_port: Option<u16>,
  #[arg(long, help = "Store ACME TLS certificates in <ACME_CACHE>.")]
  acme_cache: Option<PathBuf>,
  #[arg(long, help = "Provide ACME contact <ACME_CONTACT>.")]
  acme_contact: Vec<String>,
  #[arg(long, help = "Serve HTTP traffic on <HTTP_PORT>.")]
  http: bool,
  #[arg(long, help = "Serve HTTPS traffic on <HTTPS_PORT>.")]
  https: bool,
  #[arg(long, help = "Redirect HTTP traffic to HTTPS.")]
  redirect_http_to_https: bool,
  #[arg(long, short = 'j', help = "Enable JSON API.")]
  pub(crate) enable_json_api: bool,
  #[arg(
    long,
    help = "Decompress encoded content. Currently only supports brotli. Be careful using this on production instances. A decompressed inscription may be arbitrarily large, making decompression a DoS vector."
  )]
  pub(crate) decompress: bool,
  #[arg(long, alias = "nosync", help = "Do not update the index.")]
  no_sync: bool,
}

impl Server {
  pub(crate) fn run(self, options: Options, index: Arc<Index>, handle: Handle) -> SubcommandResult {
    Runtime::new()?.block_on(async {
      let index_clone = index.clone();

      let index_thread = thread::spawn(move || loop {
        if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
          break;
        }
        if !self.no_sync {
          if let Err(error) = index_clone.update() {
            log::warn!("Updating index: {error}");
          }
        }
        thread::sleep(Duration::from_millis(5000));
      });
      INDEXER.lock().unwrap().replace(index_thread);

      let config = Arc::new(options.load_config()?);
      let acme_domains = self.acme_domains()?;

      let server_config = Arc::new(ServerConfig {
        chain: options.chain(),
        csp_origin: self.csp_origin.clone(),
        domain: acme_domains.first().cloned(),
        index_sats: index.has_sat_index(),
        is_json_api_enabled: self.enable_json_api,
        decompress: self.decompress,
      });

      let router = Router::new()
        .route("/", get(Self::home))
        .route("/block/:query", get(Self::block))
        .route("/blockcount", get(Self::block_count))
        .route("/blockhash", get(Self::block_hash))
        .route("/blockhash/:height", get(Self::block_hash_from_height))
        .route("/blockheight", get(Self::block_height))
        .route("/blocks", get(Self::blocks))
        .route("/blocktime", get(Self::block_time))
        .route("/bounties", get(Self::bounties))
        .route("/children/:inscription_id", get(Self::children))
        .route(
          "/children/:inscription_id/:page",
          get(Self::children_paginated),
        )
        .route("/clock", get(Self::clock))
        .route("/collections", get(Self::collections))
        .route("/collections/:page", get(Self::collections_paginated))
        .route("/content/:inscription_id", get(Self::content))
        .route("/faq", get(Self::faq))
        .route("/favicon.ico", get(Self::favicon))
        .route("/feed.xml", get(Self::feed))
        .route("/input/:block/:transaction/:input", get(Self::input))
        .route("/inscription/:inscription_query", get(Self::inscription))
        .route("/inscriptions", get(Self::inscriptions))
        .route("/inscriptions/:page", get(Self::inscriptions_paginated))
        .route(
          "/inscriptions/block/:height",
          get(Self::inscriptions_in_block),
        )
        .route(
          "/inscriptions/block/:height/:page",
          get(Self::inscriptions_in_block_paginated),
        )
        .route("/install.sh", get(Self::install_script))
        .route("/ordinal/:sat", get(Self::ordinal))
        .route("/output/:output", get(Self::output))
        .route("/preview/:inscription_id", get(Self::preview))
        .route("/r/blockhash", get(Self::block_hash_json))
        .route(
          "/r/blockhash/:height",
          get(Self::block_hash_from_height_json),
        )
        .route("/r/blockheight", get(Self::block_height))
        .route("/r/blocktime", get(Self::block_time))
        .route("/r/children/:inscription_id", get(Self::children_recursive))
        .route(
          "/r/children/:inscription_id/:page",
          get(Self::children_recursive_paginated),
        )
        .route("/r/metadata/:inscription_id", get(Self::metadata))
        .route("/r/sat/:sat_number", get(Self::sat_inscriptions))
        .route(
          "/r/sat/:sat_number/:page",
          get(Self::sat_inscriptions_paginated),
        )
        .route(
          "/r/sat/:sat_number/at/:index",
          get(Self::sat_inscription_at_index),
        )
        .route("/range/:start/:end", get(Self::range))
        .route("/rare.txt", get(Self::rare_txt))
        .route("/rune/:rune", get(Self::rune))
        .route("/runes", get(Self::runes))
        .route("/sat/:sat", get(Self::sat))
        .route("/search", get(Self::search_by_query))
        .route("/search/*query", get(Self::search_by_path))
        .route("/static/*path", get(Self::static_asset))
        .route("/status", get(Self::status))
        .route("/tx/:txid", get(Self::transaction))
        // ---- Ordzaar routes ----
        .route("/inscriptions", post(Self::ordzaar_inscriptions_from_ids))
        .route("/ordinals/:outpoint", get(Self::ordzaar_ordinals_from_outpoint))
        // ---- Ordzaar routes ----
        .layer(Extension(index))
        .layer(Extension(server_config.clone()))
        .layer(Extension(config))
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
        .layer(CompressionLayer::new())
        .with_state(server_config);

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

      Ok(Box::new(Empty {}) as Box<dyn Output>)
    })
  }

  // ---- Ordzaar methods ----
  async fn ordzaar_ordinals_from_outpoint(
    Extension(index): Extension<Arc<Index>>,
    Path(outpoint): Path<OutPoint>,
  ) -> ServerResult<Response> {
    index
      .get_transaction(outpoint.txid)?
      .ok_or_not_found(|| format!("output {outpoint}"))?
      .output
      .into_iter()
      .nth(outpoint.vout as usize)
      .ok_or_not_found(|| format!("output {outpoint}"))?;
    Ok(Json(get_ordinals(&index, outpoint)?).into_response())
  }

  async fn ordzaar_inscriptions_from_ids(
    Extension(index): Extension<Arc<Index>>,
    Json(payload): Json<InscriptionIds>,
  ) -> ServerResult<Response> {
    let mut inscriptions: Vec<InscriptionData> = Vec::new();
    for id in payload.ids {
      let entry = match index.get_inscription_entry(id)? {
        Some(entry) => entry,
        None => continue,
      };

      let satpoint = match index.get_inscription_satpoint_by_id(id)? {
        Some(satpoint) => satpoint,
        None => continue,
      };

      inscriptions.push(InscriptionData::new(
        entry.fee,
        entry.height,
        id,
        entry.inscription_number,
        entry.sequence_number,
        entry.sat,
        satpoint,
        timestamp(entry.timestamp),
      ));
    }
    Ok(Json(inscriptions).into_response())
  }
  // ---- Ordzaar methods ----

  fn spawn(
    &self,
    router: Router,
    handle: Handle,
    port: u16,
    config: SpawnConfig,
  ) -> Result<task::JoinHandle<io::Result<()>>> {
    let address = match &self.address {
      Some(address) => address.as_str(),
      None => {
        if cfg!(test) || integration_test() {
          "127.0.0.1"
        } else {
          "0.0.0.0"
        }
      }
    };

    let addr = (address, port)
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

  fn acme_cache(acme_cache: Option<&PathBuf>, options: &Options) -> PathBuf {
    acme_cache
      .unwrap_or(&options.data_dir().join("acme-cache"))
      .to_path_buf()
  }

  fn acme_domains(&self) -> Result<Vec<String>> {
    if !self.acme_domain.is_empty() {
      Ok(self.acme_domain.clone())
    } else {
      Ok(vec![
        System::host_name().ok_or(anyhow!("no hostname found"))?
      ])
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
      ))))
      .directory(if cfg!(test) {
        LETS_ENCRYPT_STAGING_DIRECTORY
      } else {
        LETS_ENCRYPT_PRODUCTION_DIRECTORY
      });

    let mut state = config.state();

    let mut server_config = rustls::ServerConfig::builder()
      .with_no_client_auth()
      .with_cert_resolver(state.resolver());

    server_config.alpn_protocols = vec!["h2".into(), "http/1.1".into()];

    let acceptor = state.axum_acceptor(Arc::new(server_config));

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
    task::block_in_place(|| {
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
    })
  }

  async fn sat(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(DeserializeFromStr(sat)): Path<DeserializeFromStr<Sat>>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      let inscriptions = index.get_inscription_ids_by_sat(sat)?;
      let satpoint = index.rare_sat_satpoint(sat)?.or_else(|| {
        inscriptions.first().and_then(|&first_inscription_id| {
          index
            .get_inscription_satpoint_by_id(first_inscription_id)
            .ok()
            .flatten()
        })
      });
      let blocktime = index.block_time(sat.height())?;
      Ok(if accept_json {
        Json(SatJson {
          number: sat.0,
          decimal: sat.decimal().to_string(),
          degree: sat.degree().to_string(),
          name: sat.name(),
          block: sat.height().0,
          cycle: sat.cycle(),
          epoch: sat.epoch().0,
          period: sat.period(),
          offset: sat.third(),
          rarity: sat.rarity(),
          percentile: sat.percentile(),
          satpoint,
          timestamp: blocktime.timestamp().timestamp(),
          inscriptions,
        })
        .into_response()
      } else {
        SatHtml {
          sat,
          satpoint,
          blocktime,
          inscriptions,
        }
        .page(server_config)
        .into_response()
      })
    })
  }

  async fn ordinal(Path(sat): Path<String>) -> Redirect {
    Redirect::to(&format!("/sat/{sat}"))
  }

  async fn output(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(outpoint): Path<OutPoint>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      let list = index.list(outpoint)?;

      let output = if outpoint == OutPoint::null() || outpoint == unbound_outpoint() {
        let mut value = 0;

        if let Some(List::Unspent(ranges)) = &list {
          for (start, end) in ranges {
            value += end - start;
          }
        }

        TxOut {
          value,
          script_pubkey: ScriptBuf::new(),
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

      let runes = index.get_rune_balances_for_outpoint(outpoint)?;

      Ok(if accept_json {
        Json(OutputJson::new(
          outpoint,
          list,
          server_config.chain,
          output,
          inscriptions,
          runes
            .into_iter()
            .map(|(spaced_rune, pile)| (spaced_rune.rune, pile.amount))
            .collect(),
        ))
        .into_response()
      } else {
        OutputHtml {
          outpoint,
          inscriptions,
          list,
          chain: server_config.chain,
          output,
          runes,
        }
        .page(server_config)
        .into_response()
      })
    })
  }

  async fn range(
    Extension(server_config): Extension<Arc<ServerConfig>>,
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
      Ordering::Less => Ok(RangeHtml { start, end }.page(server_config)),
    }
  }

  async fn rare_txt(Extension(index): Extension<Arc<Index>>) -> ServerResult<RareTxt> {
    task::block_in_place(|| Ok(RareTxt(index.rare_sat_satpoints()?)))
  }

  async fn rune(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(DeserializeFromStr(spaced_rune)): Path<DeserializeFromStr<SpacedRune>>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      if !index.has_rune_index() {
        return Err(ServerError::NotFound(
          "this server has no rune index".to_string(),
        ));
      }

      let (id, entry, parent) = index
        .rune(spaced_rune.rune)?
        .ok_or_not_found(|| format!("rune {spaced_rune}"))?;

      Ok(if accept_json {
        Json(RuneJson { entry, id, parent }).into_response()
      } else {
        RuneHtml { entry, id, parent }
          .page(server_config)
          .into_response()
      })
    })
  }

  async fn runes(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      Ok(if accept_json {
        Json(RunesJson {
          entries: index.runes()?,
        })
        .into_response()
      } else {
        RunesHtml {
          entries: index.runes()?,
        }
        .page(server_config)
        .into_response()
      })
    })
  }

  async fn home(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
  ) -> ServerResult<PageHtml<HomeHtml>> {
    task::block_in_place(|| {
      Ok(
        HomeHtml {
          inscriptions: index.get_home_inscriptions()?,
        }
        .page(server_config),
      )
    })
  }

  async fn blocks(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
  ) -> ServerResult<PageHtml<BlocksHtml>> {
    task::block_in_place(|| {
      let blocks = index.blocks(100)?;
      let mut featured_blocks = BTreeMap::new();
      for (height, hash) in blocks.iter().take(5) {
        let (inscriptions, _total_num) =
          index.get_highest_paying_inscriptions_in_block(*height, 8)?;

        featured_blocks.insert(*hash, inscriptions);
      }

      Ok(BlocksHtml::new(blocks, featured_blocks).page(server_config))
    })
  }

  async fn install_script() -> Redirect {
    Redirect::to("https://raw.githubusercontent.com/ordinals/ord/master/install.sh")
  }

  async fn block(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(DeserializeFromStr(query)): Path<DeserializeFromStr<BlockQuery>>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
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

          (block, u32::try_from(info.height).unwrap())
        }
      };

      Ok(if accept_json {
        let inscriptions = index.get_inscriptions_in_block(height)?;
        Json(BlockJson::new(
          block,
          Height(height),
          Self::index_height(&index)?,
          inscriptions,
        ))
        .into_response()
      } else {
        let (featured_inscriptions, total_num) =
          index.get_highest_paying_inscriptions_in_block(height, 8)?;
        BlockHtml::new(
          block,
          Height(height),
          Self::index_height(&index)?,
          total_num,
          featured_inscriptions,
        )
        .page(server_config)
        .into_response()
      })
    })
  }

  async fn transaction(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(txid): Path<Txid>,
  ) -> ServerResult<PageHtml<TransactionHtml>> {
    task::block_in_place(|| {
      let transaction = index
        .get_transaction(txid)?
        .ok_or_not_found(|| format!("transaction {txid}"))?;

      let inscription_count = index.inscription_count(txid)?;

      let blockhash = index.get_transaction_blockhash(txid)?;

      Ok(
        TransactionHtml {
          blockhash,
          transaction,
          txid,
          inscription_count,
          chain: server_config.chain,
          etching: index.get_etching(txid)?,
        }
        .page(server_config),
      )
    })
  }

  async fn metadata(
    Extension(index): Extension<Arc<Index>>,
    Path(inscription_id): Path<InscriptionId>,
  ) -> ServerResult<Json<String>> {
    task::block_in_place(|| {
      let metadata = index
        .get_inscription_by_id(inscription_id)?
        .ok_or_not_found(|| format!("inscription {inscription_id}"))?
        .metadata
        .ok_or_not_found(|| format!("inscription {inscription_id} metadata"))?;

      Ok(Json(hex::encode(metadata)))
    })
  }

  async fn status(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      Ok(if accept_json {
        Json(index.status()?).into_response()
      } else {
        index.status()?.page(server_config).into_response()
      })
    })
  }

  async fn search_by_query(
    Extension(index): Extension<Arc<Index>>,
    Query(search): Query<Search>,
  ) -> ServerResult<Redirect> {
    Self::search(index, search.query).await
  }

  async fn search_by_path(
    Extension(index): Extension<Arc<Index>>,
    Path(search): Path<Search>,
  ) -> ServerResult<Redirect> {
    Self::search(index, search.query).await
  }

  async fn search(index: Arc<Index>, query: String) -> ServerResult<Redirect> {
    Self::search_inner(index, query).await
  }

  async fn search_inner(index: Arc<Index>, query: String) -> ServerResult<Redirect> {
    task::block_in_place(|| {
      lazy_static! {
        static ref HASH: Regex = Regex::new(r"^[[:xdigit:]]{64}$").unwrap();
        static ref INSCRIPTION_ID: Regex = Regex::new(r"^[[:xdigit:]]{64}i\d+$").unwrap();
        static ref OUTPOINT: Regex = Regex::new(r"^[[:xdigit:]]{64}:\d+$").unwrap();
        static ref RUNE: Regex = Regex::new(r"^[A-Z•.]+$").unwrap();
        static ref RUNE_ID: Regex = Regex::new(r"^[0-9]+/[0-9]+$").unwrap();
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
      } else if RUNE.is_match(query) {
        Ok(Redirect::to(&format!("/rune/{query}")))
      } else if RUNE_ID.is_match(query) {
        let id = query
          .parse::<RuneId>()
          .map_err(|err| ServerError::BadRequest(err.to_string()))?;

        let rune = index.get_rune_by_id(id)?.ok_or_not_found(|| "rune ID")?;

        Ok(Redirect::to(&format!("/rune/{rune}")))
      } else {
        Ok(Redirect::to(&format!("/sat/{query}")))
      }
    })
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
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      let mut builder = rss::ChannelBuilder::default();

      let chain = server_config.chain;
      match chain {
        Chain::Mainnet => builder.title("Inscriptions".to_string()),
        _ => builder.title(format!("Inscriptions – {chain:?}")),
      };

      builder.generator(Some("ord".to_string()));

      for (number, id) in index.get_feed_inscriptions(300)? {
        builder.item(
          rss::ItemBuilder::default()
            .title(Some(format!("Inscription {number}")))
            .link(Some(format!("/inscription/{id}")))
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
    })
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
    task::block_in_place(|| Ok(index.block_count()?.to_string()))
  }

  async fn block_height(Extension(index): Extension<Arc<Index>>) -> ServerResult<String> {
    task::block_in_place(|| {
      Ok(
        index
          .block_height()?
          .ok_or_not_found(|| "blockheight")?
          .to_string(),
      )
    })
  }

  async fn block_hash(Extension(index): Extension<Arc<Index>>) -> ServerResult<String> {
    task::block_in_place(|| {
      Ok(
        index
          .block_hash(None)?
          .ok_or_not_found(|| "blockhash")?
          .to_string(),
      )
    })
  }

  async fn block_hash_json(Extension(index): Extension<Arc<Index>>) -> ServerResult<Json<String>> {
    task::block_in_place(|| {
      Ok(Json(
        index
          .block_hash(None)?
          .ok_or_not_found(|| "blockhash")?
          .to_string(),
      ))
    })
  }

  async fn block_hash_from_height(
    Extension(index): Extension<Arc<Index>>,
    Path(height): Path<u32>,
  ) -> ServerResult<String> {
    task::block_in_place(|| {
      Ok(
        index
          .block_hash(Some(height))?
          .ok_or_not_found(|| "blockhash")?
          .to_string(),
      )
    })
  }

  async fn block_hash_from_height_json(
    Extension(index): Extension<Arc<Index>>,
    Path(height): Path<u32>,
  ) -> ServerResult<Json<String>> {
    task::block_in_place(|| {
      Ok(Json(
        index
          .block_hash(Some(height))?
          .ok_or_not_found(|| "blockhash")?
          .to_string(),
      ))
    })
  }

  async fn block_time(Extension(index): Extension<Arc<Index>>) -> ServerResult<String> {
    task::block_in_place(|| {
      Ok(
        index
          .block_time(index.block_height()?.ok_or_not_found(|| "blocktime")?)?
          .unix_timestamp()
          .to_string(),
      )
    })
  }

  async fn input(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(path): Path<(u32, usize, usize)>,
  ) -> ServerResult<PageHtml<InputHtml>> {
    task::block_in_place(|| {
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

      Ok(InputHtml { path, input }.page(server_config))
    })
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
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Path(inscription_id): Path<InscriptionId>,
    accept_encoding: AcceptEncoding,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      if config.is_hidden(inscription_id) {
        return Ok(PreviewUnknownHtml.into_response());
      }

      let mut inscription = index
        .get_inscription_by_id(inscription_id)?
        .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

      if let Some(delegate) = inscription.delegate() {
        inscription = index
          .get_inscription_by_id(delegate)?
          .ok_or_not_found(|| format!("delegate {inscription_id}"))?
      }

      Ok(
        Self::content_response(inscription, accept_encoding, &server_config)?
          .ok_or_not_found(|| format!("inscription {inscription_id} content"))?
          .into_response(),
      )
    })
  }

  fn content_response(
    inscription: Inscription,
    accept_encoding: AcceptEncoding,
    server_config: &ServerConfig,
  ) -> ServerResult<Option<(HeaderMap, Vec<u8>)>> {
    let mut headers = HeaderMap::new();

    match &server_config.csp_origin {
      None => {
        headers.insert(
          header::CONTENT_SECURITY_POLICY,
          HeaderValue::from_static("default-src 'self' 'unsafe-eval' 'unsafe-inline' data: blob:"),
        );
        headers.append(
          header::CONTENT_SECURITY_POLICY,
          HeaderValue::from_static("default-src *:*/content/ *:*/blockheight *:*/blockhash *:*/blockhash/ *:*/blocktime *:*/r/ 'unsafe-eval' 'unsafe-inline' data: blob:"),
        );
      }
      Some(origin) => {
        let csp = format!("default-src {origin}/content/ {origin}/blockheight {origin}/blockhash {origin}/blockhash/ {origin}/blocktime {origin}/r/ 'unsafe-eval' 'unsafe-inline' data: blob:");
        headers.insert(
          header::CONTENT_SECURITY_POLICY,
          HeaderValue::from_str(&csp).map_err(|err| ServerError::Internal(Error::from(err)))?,
        );
      }
    }

    headers.insert(
      header::CACHE_CONTROL,
      HeaderValue::from_static("public, max-age=31536000, immutable"),
    );

    headers.insert(
      header::CONTENT_TYPE,
      inscription
        .content_type()
        .and_then(|content_type| content_type.parse().ok())
        .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );

    if let Some(content_encoding) = inscription.content_encoding() {
      if accept_encoding.is_acceptable(&content_encoding) {
        headers.insert(header::CONTENT_ENCODING, content_encoding);
      } else if server_config.decompress && content_encoding == "br" {
        let Some(body) = inscription.into_body() else {
          return Ok(None);
        };

        let mut decompressed = Vec::new();

        Decompressor::new(body.as_slice(), 4096)
          .read_to_end(&mut decompressed)
          .map_err(|err| ServerError::Internal(err.into()))?;

        return Ok(Some((headers, decompressed)));
      } else {
        return Err(ServerError::NotAcceptable {
          accept_encoding,
          content_encoding,
        });
      }
    }

    let Some(body) = inscription.into_body() else {
      return Ok(None);
    };

    Ok(Some((headers, body)))
  }

  async fn preview(
    Extension(index): Extension<Arc<Index>>,
    Extension(config): Extension<Arc<Config>>,
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Path(inscription_id): Path<InscriptionId>,
    accept_encoding: AcceptEncoding,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      if config.is_hidden(inscription_id) {
        return Ok(PreviewUnknownHtml.into_response());
      }

      let mut inscription = index
        .get_inscription_by_id(inscription_id)?
        .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

      if let Some(delegate) = inscription.delegate() {
        inscription = index
          .get_inscription_by_id(delegate)?
          .ok_or_not_found(|| format!("delegate {inscription_id}"))?
      }

      match inscription.media() {
        Media::Audio => Ok(PreviewAudioHtml { inscription_id }.into_response()),
        Media::Code(language) => Ok(
          (
            [(
              header::CONTENT_SECURITY_POLICY,
              "script-src-elem 'self' https://cdn.jsdelivr.net",
            )],
            PreviewCodeHtml {
              inscription_id,
              language,
            },
          )
            .into_response(),
        ),
        Media::Font => Ok(
          (
            [(
              header::CONTENT_SECURITY_POLICY,
              "script-src-elem 'self'; style-src 'self' 'unsafe-inline';",
            )],
            PreviewFontHtml { inscription_id },
          )
            .into_response(),
        ),
        Media::Iframe => Ok(
          Self::content_response(inscription, accept_encoding, &server_config)?
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
        Media::Markdown => Ok(
          (
            [(
              header::CONTENT_SECURITY_POLICY,
              "script-src-elem 'self' https://cdn.jsdelivr.net",
            )],
            PreviewMarkdownHtml { inscription_id },
          )
            .into_response(),
        ),
        Media::Model => Ok(
          (
            [(
              header::CONTENT_SECURITY_POLICY,
              "script-src-elem 'self' https://ajax.googleapis.com",
            )],
            PreviewModelHtml { inscription_id },
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
        Media::Text => Ok(PreviewTextHtml { inscription_id }.into_response()),
        Media::Unknown => Ok(PreviewUnknownHtml.into_response()),
        Media::Video => Ok(PreviewVideoHtml { inscription_id }.into_response()),
      }
    })
  }

  async fn inscription(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(DeserializeFromStr(query)): Path<DeserializeFromStr<InscriptionQuery>>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      let info = Index::inscription_info(&index, query)?
        .ok_or_not_found(|| format!("inscription {query}"))?;

      Ok(if accept_json {
        Json(InscriptionJson {
          inscription_id: info.entry.id,
          charms: Charm::ALL
            .iter()
            .filter(|charm| charm.is_set(info.charms))
            .map(|charm| charm.title().into())
            .collect(),
          children: info.children,
          inscription_number: info.entry.inscription_number,
          genesis_height: info.entry.height,
          parent: info.parent,
          genesis_fee: info.entry.fee,
          output_value: info.output.as_ref().map(|o| o.value),
          address: info
            .output
            .as_ref()
            .and_then(|o| {
              server_config
                .chain
                .address_from_script(&o.script_pubkey)
                .ok()
            })
            .map(|address| address.to_string()),
          sat: info.entry.sat,
          satpoint: info.satpoint,
          content_type: info.inscription.content_type().map(|s| s.to_string()),
          content_length: info.inscription.content_length(),
          timestamp: timestamp(info.entry.timestamp).timestamp(),
          previous: info.previous,
          next: info.next,
          rune: info.rune,

          // ---- Ordzaar ----
          inscription_sequence: info.entry.sequence_number,
          // ---- Ordzaar ----
        })
        .into_response()
      } else {
        InscriptionHtml {
          chain: server_config.chain,
          charms: info.charms,
          children: info.children,
          genesis_fee: info.entry.fee,
          genesis_height: info.entry.height,
          inscription: info.inscription,
          inscription_id: info.entry.id,
          inscription_number: info.entry.inscription_number,
          next: info.next,
          output: info.output,
          parent: info.parent,
          previous: info.previous,
          rune: info.rune,
          sat: info.entry.sat,
          satpoint: info.satpoint,
          timestamp: timestamp(info.entry.timestamp),
        }
        .page(server_config)
        .into_response()
      })
    })
  }

  async fn collections(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
  ) -> ServerResult<Response> {
    Self::collections_paginated(Extension(server_config), Extension(index), Path(0)).await
  }

  async fn collections_paginated(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(page_index): Path<usize>,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      let (collections, more_collections) = index.get_collections_paginated(100, page_index)?;

      let prev = page_index.checked_sub(1);

      let next = more_collections.then_some(page_index + 1);

      Ok(
        CollectionsHtml {
          inscriptions: collections,
          prev,
          next,
        }
        .page(server_config)
        .into_response(),
      )
    })
  }

  async fn children(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(inscription_id): Path<InscriptionId>,
  ) -> ServerResult<Response> {
    Self::children_paginated(
      Extension(server_config),
      Extension(index),
      Path((inscription_id, 0)),
    )
    .await
  }

  async fn children_paginated(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path((parent, page)): Path<(InscriptionId, usize)>,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      let entry = index
        .get_inscription_entry(parent)?
        .ok_or_not_found(|| format!("inscription {parent}"))?;

      let parent_number = entry.inscription_number;

      let (children, more_children) =
        index.get_children_by_sequence_number_paginated(entry.sequence_number, 100, page)?;

      let prev_page = page.checked_sub(1);

      let next_page = more_children.then_some(page + 1);

      Ok(
        ChildrenHtml {
          parent,
          parent_number,
          children,
          prev_page,
          next_page,
        }
        .page(server_config)
        .into_response(),
      )
    })
  }

  async fn children_recursive(
    Extension(index): Extension<Arc<Index>>,
    Path(inscription_id): Path<InscriptionId>,
  ) -> ServerResult<Response> {
    Self::children_recursive_paginated(Extension(index), Path((inscription_id, 0))).await
  }

  async fn children_recursive_paginated(
    Extension(index): Extension<Arc<Index>>,
    Path((parent, page)): Path<(InscriptionId, usize)>,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      let parent_sequence_number = index
        .get_inscription_entry(parent)?
        .ok_or_not_found(|| format!("inscription {parent}"))?
        .sequence_number;

      let (ids, more) =
        index.get_children_by_sequence_number_paginated(parent_sequence_number, 100, page)?;

      Ok(Json(ChildrenJson { ids, more, page }).into_response())
    })
  }

  async fn inscriptions(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    accept_json: AcceptJson,
  ) -> ServerResult<Response> {
    Self::inscriptions_paginated(
      Extension(server_config),
      Extension(index),
      Path(0),
      accept_json,
    )
    .await
  }

  async fn inscriptions_paginated(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(page_index): Path<u32>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      let (inscriptions, more) = index.get_inscriptions_paginated(100, page_index)?;

      let prev = page_index.checked_sub(1);

      let next = more.then_some(page_index + 1);

      Ok(if accept_json {
        Json(InscriptionsJson {
          inscriptions,
          page_index,
          more,
        })
        .into_response()
      } else {
        InscriptionsHtml {
          inscriptions,
          next,
          prev,
        }
        .page(server_config)
        .into_response()
      })
    })
  }

  async fn inscriptions_in_block(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path(block_height): Path<u32>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult<Response> {
    Self::inscriptions_in_block_paginated(
      Extension(server_config),
      Extension(index),
      Path((block_height, 0)),
      AcceptJson(accept_json),
    )
    .await
  }

  async fn inscriptions_in_block_paginated(
    Extension(server_config): Extension<Arc<ServerConfig>>,
    Extension(index): Extension<Arc<Index>>,
    Path((block_height, page_index)): Path<(u32, u32)>,
    AcceptJson(accept_json): AcceptJson,
  ) -> ServerResult<Response> {
    task::block_in_place(|| {
      let page_size = 100;

      let page_index_usize = usize::try_from(page_index).unwrap_or(usize::MAX);
      let page_size_usize = usize::try_from(page_size).unwrap_or(usize::MAX);

      let mut inscriptions = index
        .get_inscriptions_in_block(block_height)?
        .into_iter()
        .skip(page_index_usize.saturating_mul(page_size_usize))
        .take(page_size_usize.saturating_add(1))
        .collect::<Vec<InscriptionId>>();

      let more = inscriptions.len() > page_size_usize;

      if more {
        inscriptions.pop();
      }

      Ok(if accept_json {
        Json(InscriptionsJson {
          inscriptions,
          page_index,
          more,
        })
        .into_response()
      } else {
        InscriptionsBlockHtml::new(
          block_height,
          index.block_height()?.unwrap_or(Height(0)).n(),
          inscriptions,
          more,
          page_index,
        )?
        .page(server_config)
        .into_response()
      })
    })
  }

  async fn sat_inscriptions(
    Extension(index): Extension<Arc<Index>>,
    Path(sat): Path<u64>,
  ) -> ServerResult<Json<SatInscriptionsJson>> {
    Self::sat_inscriptions_paginated(Extension(index), Path((sat, 0))).await
  }

  async fn sat_inscriptions_paginated(
    Extension(index): Extension<Arc<Index>>,
    Path((sat, page)): Path<(u64, u64)>,
  ) -> ServerResult<Json<SatInscriptionsJson>> {
    task::block_in_place(|| {
      if !index.has_sat_index() {
        return Err(ServerError::NotFound(
          "this server has no sat index".to_string(),
        ));
      }

      let (ids, more) = index.get_inscription_ids_by_sat_paginated(Sat(sat), 100, page)?;

      Ok(Json(SatInscriptionsJson { ids, more, page }))
    })
  }

  async fn sat_inscription_at_index(
    Extension(index): Extension<Arc<Index>>,
    Path((DeserializeFromStr(sat), inscription_index)): Path<(DeserializeFromStr<Sat>, isize)>,
  ) -> ServerResult<Json<SatInscriptionJson>> {
    task::block_in_place(|| {
      if !index.has_sat_index() {
        return Err(ServerError::NotFound(
          "this server has no sat index".to_string(),
        ));
      }

      let id = index.get_inscription_id_by_sat_indexed(sat, inscription_index)?;

      Ok(Json(SatInscriptionJson { id }))
    })
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
  use {
    super::*,
    crate::runes::{Edict, Etching, Rune, Runestone},
    reqwest::Url,
    serde::de::DeserializeOwned,
    std::net::TcpListener,
  };

  const RUNE: u128 = 99246114928149462;

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
          .network(bitcoin::network::constants::Network::Regtest)
          .build(),
        None,
        &["--chain", "regtest"],
        &[],
      )
    }

    fn new_with_regtest_with_json_api() -> Self {
      Self::new_server(
        test_bitcoincore_rpc::builder()
          .network(bitcoin::network::constants::Network::Regtest)
          .build(),
        None,
        &["--chain", "regtest"],
        &["--enable-json-api"],
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

    fn new_with_regtest_with_index_runes() -> Self {
      Self::new_server(
        test_bitcoincore_rpc::builder()
          .network(bitcoin::Network::Regtest)
          .build(),
        None,
        &["--chain", "regtest", "--index-runes"],
        &["--enable-json-api"],
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

    pub(crate) fn get_json<T: DeserializeOwned>(&self, path: impl AsRef<str>) -> T {
      if let Err(error) = self.index.update() {
        log::error!("{error}");
      }

      let client = reqwest::blocking::Client::new();

      let response = client
        .get(self.join_url(path.as_ref()))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .unwrap();

      assert_eq!(response.status(), StatusCode::OK);

      response.json().unwrap()
    }

    fn join_url(&self, url: &str) -> Url {
      self.url.join(url).unwrap()
    }

    fn assert_response(&self, path: impl AsRef<str>, status: StatusCode, expected_response: &str) {
      let response = self.get(path);
      assert_eq!(response.status(), status, "{}", response.text().unwrap());
      pretty_assert_eq!(response.text().unwrap(), expected_response);
    }

    #[track_caller]
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

    #[track_caller]
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
      .display()
      .to_string();
    assert_eq!(acme_cache, "bar")
  }

  #[test]
  fn acme_domain_defaults_to_hostname() {
    let (_, server) = parse_server_args("ord server");
    assert_eq!(
      server.acme_domains().unwrap(),
      &[System::host_name().unwrap()]
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
      "https://raw.githubusercontent.com/ordinals/ord/master/install.sh",
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
  fn search_by_query_returns_rune() {
    TestServer::new().assert_redirect("/search?query=ABCD", "/rune/ABCD");
  }

  #[test]
  fn search_by_query_returns_spaced_rune() {
    TestServer::new().assert_redirect("/search?query=AB•CD", "/rune/AB•CD");
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
  fn search_by_path_returns_rune() {
    TestServer::new().assert_redirect("/search/ABCD", "/rune/ABCD");
  }

  #[test]
  fn search_by_path_returns_spaced_rune() {
    TestServer::new().assert_redirect("/search/AB•CD", "/rune/AB•CD");
  }

  #[test]
  fn search_by_rune_id_returns_rune() {
    let server = TestServer::new_with_regtest_with_index_runes();

    server.mine_blocks(1);

    let rune = Rune(RUNE);

    server.assert_response_regex(format!("/rune/{rune}"), StatusCode::NOT_FOUND, ".*");

    server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            rune: Some(rune),
            ..Default::default()
          }),
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_redirect("/search/2/1", "/rune/AAAAAAAAAAAAA");
    server.assert_redirect("/search?query=2/1", "/rune/AAAAAAAAAAAAA");

    server.assert_response_regex("/rune/100/200", StatusCode::NOT_FOUND, ".*");

    server.assert_response_regex(
      "/search/100000000000000000000/200000000000000000",
      StatusCode::BAD_REQUEST,
      ".*",
    );
  }

  #[test]
  fn runes_are_displayed_on_runes_page() {
    let server = TestServer::new_with_regtest_with_index_runes();

    server.mine_blocks(1);

    server.assert_response_regex(
      "/runes",
      StatusCode::OK,
      ".*<title>Runes</title>.*<h1>Runes</h1>\n<ul>\n</ul>.*",
    );

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            rune: Some(Rune(RUNE)),
            ..Default::default()
          }),
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    server.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      server.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          timestamp: 2,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      server.index.get_rune_balances().unwrap(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );

    server.assert_response_regex(
      "/runes",
      StatusCode::OK,
      ".*<title>Runes</title>.*
<h1>Runes</h1>
<ul>
  <li><a href=/rune/AAAAAAAAAAAAA>AAAAAAAAAAAAA</a></li>
</ul>.*",
    );
  }

  #[test]
  fn runes_are_displayed_on_rune_page() {
    let server = TestServer::new_with_regtest_with_index_runes();

    server.mine_blocks(1);

    let rune = Rune(RUNE);

    server.assert_response_regex(format!("/rune/{rune}"), StatusCode::NOT_FOUND, ".*");

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            rune: Some(rune),
            symbol: Some('%'),
            ..Default::default()
          }),
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    server.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      server.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune,
          supply: u128::max_value(),
          symbol: Some('%'),
          timestamp: 2,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      server.index.get_rune_balances().unwrap(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );

    server.assert_response_regex(
      format!("/rune/{rune}"),
      StatusCode::OK,
      format!(
        ".*<title>Rune AAAAAAAAAAAAA</title>.*
<h1>AAAAAAAAAAAAA</h1>
<iframe .* src=/preview/{txid}i0></iframe>
<dl>
  <dt>number</dt>
  <dd>0</dd>
  <dt>timestamp</dt>
  <dd><time>1970-01-01 00:00:02 UTC</time></dd>
  <dt>id</dt>
  <dd>2/1</dd>
  <dt>etching block height</dt>
  <dd><a href=/block/2>2</a></dd>
  <dt>etching transaction index</dt>
  <dd>1</dd>
  <dt>mints</dt>
  <dd>0</dd>
  <dt>supply</dt>
  <dd>340282366920938463463374607431768211455\u{00A0}%</dd>
  <dt>burned</dt>
  <dd>0\u{00A0}%</dd>
  <dt>divisibility</dt>
  <dd>0</dd>
  <dt>symbol</dt>
  <dd>%</dd>
  <dt>etching</dt>
  <dd><a class=monospace href=/tx/{txid}>{txid}</a></dd>
  <dt>parent</dt>
  <dd><a class=monospace href=/inscription/{txid}i0>{txid}i0</a></dd>
</dl>
.*"
      ),
    );

    server.assert_response_regex(
      format!("/inscription/{txid}i0"),
      StatusCode::OK,
      ".*
<dl>
  .*
  <dt>rune</dt>
  <dd><a href=/rune/AAAAAAAAAAAAA>AAAAAAAAAAAAA</a></dd>
</dl>
.*",
    );
  }

  #[test]
  fn runes_are_spaced() {
    let server = TestServer::new_with_regtest_with_index_runes();

    server.mine_blocks(1);

    let rune = Rune(RUNE);

    server.assert_response_regex(format!("/rune/{rune}"), StatusCode::NOT_FOUND, ".*");

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            rune: Some(rune),
            symbol: Some('%'),
            spacers: 1,
            ..Default::default()
          }),
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    server.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      server.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune,
          supply: u128::max_value(),
          symbol: Some('%'),
          timestamp: 2,
          spacers: 1,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      server.index.get_rune_balances().unwrap(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );

    server.assert_response_regex(
      format!("/rune/{rune}"),
      StatusCode::OK,
      r".*<title>Rune A•AAAAAAAAAAAA</title>.*<h1>A•AAAAAAAAAAAA</h1>.*",
    );

    server.assert_response_regex(
      format!("/inscription/{txid}i0"),
      StatusCode::OK,
      ".*<dt>rune</dt>.*<dd><a href=/rune/A•AAAAAAAAAAAA>A•AAAAAAAAAAAA</a></dd>.*",
    );

    server.assert_response_regex(
      "/runes",
      StatusCode::OK,
      ".*<li><a href=/rune/A•AAAAAAAAAAAA>A•AAAAAAAAAAAA</a></li>.*",
    );

    server.assert_response_regex(
      format!("/tx/{txid}"),
      StatusCode::OK,
      ".*
  <dt>etching</dt>
  <dd><a href=/rune/A•AAAAAAAAAAAA>A•AAAAAAAAAAAA</a></dd>
.*",
    );

    server.assert_response_regex(
      format!("/output/{txid}:0"),
      StatusCode::OK,
      ".*<tr>
        <td><a href=/rune/A•AAAAAAAAAAAA>A•AAAAAAAAAAAA</a></td>
        <td>340282366920938463463374607431768211455\u{00A0}%</td>
      </tr>.*",
    );
  }

  #[test]
  fn transactions_link_to_etching() {
    let server = TestServer::new_with_regtest_with_index_runes();

    server.mine_blocks(1);

    server.assert_response_regex(
      "/runes",
      StatusCode::OK,
      ".*<title>Runes</title>.*<h1>Runes</h1>\n<ul>\n</ul>.*",
    );

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            rune: Some(Rune(RUNE)),
            ..Default::default()
          }),
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    server.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      server.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          timestamp: 2,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      server.index.get_rune_balances().unwrap(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );

    server.assert_response_regex(
      format!("/tx/{txid}"),
      StatusCode::OK,
      ".*
  <dt>etching</dt>
  <dd><a href=/rune/AAAAAAAAAAAAA>AAAAAAAAAAAAA</a></dd>
.*",
    );
  }

  #[test]
  fn runes_are_displayed_on_output_page() {
    let server = TestServer::new_with_regtest_with_index_runes();

    server.mine_blocks(1);

    let rune = Rune(RUNE);

    server.assert_response_regex(format!("/rune/{rune}"), StatusCode::NOT_FOUND, ".*");

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Default::default())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 1,
            rune: Some(rune),
            ..Default::default()
          }),
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    server.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      server.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          divisibility: 1,
          etching: txid,
          rune,
          supply: u128::max_value(),
          timestamp: 2,
          ..Default::default()
        }
      )]
    );

    let output = OutPoint { txid, vout: 0 };

    assert_eq!(
      server.index.get_rune_balances().unwrap(),
      [(output, vec![(id, u128::max_value())])]
    );

    server.assert_response_regex(
      format!("/output/{output}"),
      StatusCode::OK,
      format!(
        ".*<title>Output {output}</title>.*<h1>Output <span class=monospace>{output}</span></h1>.*
  <dt>runes</dt>
  <dd>
    <table>
      <tr>
        <th>rune</th>
        <th>balance</th>
      </tr>
      <tr>
        <td><a href=/rune/AAAAAAAAAAAAA>AAAAAAAAAAAAA</a></td>
        <td>34028236692093846346337460743176821145.5</td>
      </tr>
    </table>
  </dd>
.*"
      ),
    );

    assert_eq!(
      server.get_json::<OutputJson>(format!("/output/{output}")),
      OutputJson {
        value: 5000000000,
        script_pubkey: String::new(),
        address: None,
        transaction: txid.to_string(),
        sat_ranges: None,
        inscriptions: Vec::new(),
        runes: vec![(Rune(RUNE), 340282366920938463463374607431768211455)]
          .into_iter()
          .collect(),
      }
    );
  }

  #[test]
  fn http_to_https_redirect_with_path() {
    TestServer::new_with_args(&[], &["--redirect-http-to-https", "--https"]).assert_redirect(
      "/sat/0",
      &format!("https://{}/sat/0", System::host_name().unwrap()),
    );
  }

  #[test]
  fn http_to_https_redirect_with_empty() {
    TestServer::new_with_args(&[], &["--redirect-http-to-https", "--https"])
      .assert_redirect("/", &format!("https://{}/", System::host_name().unwrap()));
  }

  #[test]
  fn status() {
    let test_server = TestServer::new();

    test_server.assert_response_regex(
      "/status",
      StatusCode::OK,
      ".*<h1>Status</h1>
<dl>
  <dt>chain</dt>
  <dd>mainnet</dd>
  <dt>height</dt>
  <dd>0</dd>
  <dt>inscriptions</dt>
  <dd>0</dd>
  <dt>blessed inscriptions</dt>
  <dd>0</dd>
  <dt>cursed inscriptions</dt>
  <dd>0</dd>
  <dt>runes</dt>
  <dd>0</dd>
  <dt>lost sats</dt>
  <dd>.*</dd>
  <dt>started</dt>
  <dd>.*</dd>
  <dt>uptime</dt>
  <dd>.*</dd>
  <dt>minimum rune for next block</dt>
  <dd>AAAAAAAAAAAAA</dd>
  <dt>version</dt>
  <dd>.*</dd>
  <dt>unrecoverably reorged</dt>
  <dd>false</dd>
  <dt>rune index</dt>
  <dd>false</dd>
  <dt>sat index</dt>
  <dd>false</dd>
  <dt>transaction index</dt>
  <dd>false</dd>
  <dt>git branch</dt>
  <dd>.*</dd>
  <dt>git commit</dt>
  <dd>
    <a href=https://github.com/ordinals/ord/commit/[[:xdigit:]]{40}>
      [[:xdigit:]]{40}
    </a>
  </dd>
</dl>
.*",
    );
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
  fn unbound_output_receives_unbound_inscriptions() {
    let server = TestServer::new_with_regtest_with_index_sats();

    server.mine_blocks(1);

    server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Default::default())],
      fee: 50 * 100_000_000,
      ..Default::default()
    });

    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(
        2,
        1,
        0,
        inscription("text/plain;charset=utf-8", "hello").to_witness(),
      )],
      ..Default::default()
    });

    server.mine_blocks(1);

    let inscription_id = InscriptionId { txid, index: 0 };

    server.assert_response_regex(
      format!("/inscription/{}", inscription_id),
      StatusCode::OK,
      format!(
        ".*<dl>
  <dt>id</dt>
  <dd class=monospace>{inscription_id}</dd>.*<dt>output</dt>
  <dd><a class=monospace href=/output/0000000000000000000000000000000000000000000000000000000000000000:0>0000000000000000000000000000000000000000000000000000000000000000:0</a></dd>.*"
      ),
    );

    server.assert_response_regex(
      "/output/0000000000000000000000000000000000000000000000000000000000000000:0",
      StatusCode::OK,
      ".*<h1>Output <span class=monospace>0000000000000000000000000000000000000000000000000000000000000000:0</span></h1>
<dl>
  <dt>inscriptions</dt>
  <dd class=thumbnails>
    <a href=/inscription/.*><iframe sandbox=allow-scripts scrolling=no loading=lazy src=/preview/.*></iframe></a>
  </dd>.*",
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
    let server = TestServer::new_with_regtest();

    server.mine_blocks(1);

    let mut ids = Vec::new();

    for i in 0..101 {
      let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(i + 1, 0, 0, inscription("image/png", "hello").to_witness())],
        ..Default::default()
      });
      ids.push(InscriptionId { txid, index: 0 });
      server.mine_blocks(1);
    }

    server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "{}").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_regex(
      "/",
      StatusCode::OK,
      format!(
        r".*<title>Ordinals</title>.*
<h1>Latest Inscriptions</h1>
<div class=thumbnails>
  <a href=/inscription/{}>.*</a>
  (<a href=/inscription/[[:xdigit:]]{{64}}i0>.*</a>\s*){{99}}
</div>
.*
",
        ids[100]
      ),
    );
  }

  #[test]
  fn blocks() {
    let test_server = TestServer::new();

    test_server.mine_blocks(1);

    test_server.assert_response_regex(
      "/blocks",
      StatusCode::OK,
      ".*<title>Blocks</title>.*
<h1>Blocks</h1>
<div class=block>
  <h2><a href=/block/1>Block 1</a></h2>
  <div class=thumbnails>
  </div>
</div>
<div class=block>
  <h2><a href=/block/0>Block 0</a></h2>
  <div class=thumbnails>
  </div>
</div>
</ol>.*",
    );
  }

  #[test]
  fn nav_displays_chain() {
    TestServer::new_with_regtest().assert_response_regex(
      "/",
      StatusCode::OK,
      ".*<a href=/ title=home>Ordinals<sup>regtest</sup></a>.*",
    );
  }

  #[test]
  fn blocks_block_limit() {
    let test_server = TestServer::new();

    test_server.mine_blocks(101);

    test_server.assert_response_regex(
      "/blocks",
      StatusCode::OK,
      ".*<ol start=96 reversed class=block-list>\n(  <li><a href=/block/[[:xdigit:]]{64}>[[:xdigit:]]{64}</a></li>\n){95}</ol>.*"
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
      inputs: &[(1, 0, 0, Default::default())],
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
<dl>
</dl>
<h2>1 Input</h2>
<ul>
  <li><a class=monospace href=/output/0000000000000000000000000000000000000000000000000000000000000000:4294967295>0000000000000000000000000000000000000000000000000000000000000000:4294967295</a></li>
</ul>
<h2>1 Output</h2>
<ul class=monospace>
  <li>
    <a href=/output/84aca0d43f45ac753d4744f40b2f54edec3a496b298951735d450e601386089d:0 class=monospace>
      84aca0d43f45ac753d4744f40b2f54edec3a496b298951735d450e601386089d:0
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
  fn detect_unrecoverable_reorg() {
    let test_server = TestServer::new();

    test_server.mine_blocks(21);

    test_server.assert_response_regex(
      "/status",
      StatusCode::OK,
      ".*<dt>unrecoverably reorged</dt>\n  <dd>false</dd>.*",
    );

    for _ in 0..15 {
      test_server.bitcoin_rpc_server.invalidate_tip();
    }

    test_server.bitcoin_rpc_server.mine_blocks(21);

    test_server.assert_response_regex(
      "/status",
      StatusCode::OK,
      ".*<dt>unrecoverably reorged</dt>\n  <dd>true</dd>.*",
    );
  }

  #[test]
  fn rare_with_sat_index() {
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
      StatusCode::OK,
      "sat\tsatpoint
",
    );
  }

  #[test]
  fn show_rare_txt_in_header_with_sat_index() {
    TestServer::new_with_sat_index().assert_response_regex(
      "/",
      StatusCode::OK,
      ".*
      <a href=/clock title=clock>.*</a>
      <a href=/rare.txt title=rare>.*</a>.*",
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
      <a href=/clock title=clock>.*</a>
      <a href=https://docs.ordinals.com/.*",
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

    thread::sleep(Duration::from_millis(100));
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
      inputs: &[(1, 0, 0, Default::default())],
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
      inputs: &[(1, 0, 0, Default::default())],
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
      Server::content_response(
        Inscription::new(Some("text/plain".as_bytes().to_vec()), None),
        AcceptEncoding::default(),
        &ServerConfig::default(),
      )
      .unwrap(),
      None
    );
  }

  #[test]
  fn content_response_with_content() {
    let (headers, body) = Server::content_response(
      Inscription::new(Some("text/plain".as_bytes().to_vec()), Some(vec![1, 2, 3])),
      AcceptEncoding::default(),
      &ServerConfig::default(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(headers["content-type"], "text/plain");
    assert_eq!(body, vec![1, 2, 3]);
  }

  #[test]
  fn content_security_policy_no_origin() {
    let (headers, _) = Server::content_response(
      Inscription::new(Some("text/plain".as_bytes().to_vec()), Some(vec![1, 2, 3])),
      AcceptEncoding::default(),
      &ServerConfig::default(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
      headers["content-security-policy"],
      HeaderValue::from_static("default-src 'self' 'unsafe-eval' 'unsafe-inline' data: blob:")
    );
  }

  #[test]
  fn content_security_policy_with_origin() {
    let (headers, _) = Server::content_response(
      Inscription::new(Some("text/plain".as_bytes().to_vec()), Some(vec![1, 2, 3])),
      AcceptEncoding::default(),
      &ServerConfig {
        csp_origin: Some("https://ordinals.com".into()),
        ..Default::default()
      },
    )
    .unwrap()
    .unwrap();

    assert_eq!(headers["content-security-policy"], HeaderValue::from_static("default-src https://ordinals.com/content/ https://ordinals.com/blockheight https://ordinals.com/blockhash https://ordinals.com/blockhash/ https://ordinals.com/blocktime https://ordinals.com/r/ 'unsafe-eval' 'unsafe-inline' data: blob:"));
  }

  #[test]
  fn code_preview() {
    let server = TestServer::new_with_regtest();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(
        1,
        0,
        0,
        inscription("text/javascript", "hello").to_witness(),
      )],
      ..Default::default()
    });
    let inscription_id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/preview/{inscription_id}"),
      StatusCode::OK,
      format!(r".*<html lang=en data-inscription={inscription_id} data-language=javascript>.*"),
    );
  }

  #[test]
  fn content_response_no_content_type() {
    let (headers, body) = Server::content_response(
      Inscription::new(None, Some(Vec::new())),
      AcceptEncoding::default(),
      &ServerConfig::default(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(headers["content-type"], "application/octet-stream");
    assert!(body.is_empty());
  }

  #[test]
  fn content_response_bad_content_type() {
    let (headers, body) = Server::content_response(
      Inscription::new(Some("\n".as_bytes().to_vec()), Some(Vec::new())),
      AcceptEncoding::default(),
      &ServerConfig::default(),
    )
    .unwrap()
    .unwrap();

    assert_eq!(headers["content-type"], "application/octet-stream");
    assert!(body.is_empty());
  }

  #[test]
  fn text_preview() {
    let server = TestServer::new_with_regtest();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(
        1,
        0,
        0,
        inscription("text/plain;charset=utf-8", "hello").to_witness(),
      )],
      ..Default::default()
    });

    let inscription_id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_csp(
      format!("/preview/{}", inscription_id),
      StatusCode::OK,
      "default-src 'self'",
      format!(".*<html lang=en data-inscription={}>.*", inscription_id),
    );
  }

  #[test]
  fn audio_preview() {
    let server = TestServer::new_with_regtest();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("audio/flac", "hello").to_witness())],
      ..Default::default()
    });
    let inscription_id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/preview/{inscription_id}"),
      StatusCode::OK,
      format!(r".*<audio .*>\s*<source src=/content/{inscription_id}>.*"),
    );
  }

  #[test]
  fn font_preview() {
    let server = TestServer::new_with_regtest();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("font/ttf", "hello").to_witness())],
      ..Default::default()
    });
    let inscription_id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/preview/{inscription_id}"),
      StatusCode::OK,
      format!(r".*src: url\(/content/{inscription_id}\).*"),
    );
  }

  #[test]
  fn pdf_preview() {
    let server = TestServer::new_with_regtest();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(
        1,
        0,
        0,
        inscription("application/pdf", "hello").to_witness(),
      )],
      ..Default::default()
    });
    let inscription_id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/preview/{inscription_id}"),
      StatusCode::OK,
      format!(r".*<canvas data-inscription={inscription_id}></canvas>.*"),
    );
  }

  #[test]
  fn markdown_preview() {
    let server = TestServer::new_with_regtest();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/markdown", "hello").to_witness())],
      ..Default::default()
    });
    let inscription_id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/preview/{inscription_id}"),
      StatusCode::OK,
      format!(r".*<html lang=en data-inscription={inscription_id}>.*"),
    );
  }

  #[test]
  fn image_preview() {
    let server = TestServer::new_with_regtest();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("image/png", "hello").to_witness())],
      ..Default::default()
    });
    let inscription_id = InscriptionId { txid, index: 0 };

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
      inputs: &[(
        1,
        0,
        0,
        inscription("text/html;charset=utf-8", "hello").to_witness(),
      )],
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_csp(
      format!("/preview/{}", InscriptionId { txid, index: 0 }),
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
      inputs: &[(1, 0, 0, inscription("text/foo", "hello").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_csp(
      format!("/preview/{}", InscriptionId { txid, index: 0 }),
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
      inputs: &[(1, 0, 0, inscription("video/webm", "hello").to_witness())],
      ..Default::default()
    });
    let inscription_id = InscriptionId { txid, index: 0 };

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
      inputs: &[(1, 0, 0, inscription("text/foo", "hello").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{}", InscriptionId { txid, index: 0 }),
      StatusCode::OK,
      ".*<title>Inscription 0</title>.*",
    );
  }

  #[test]
  fn inscription_page_has_sat_when_sats_are_tracked() {
    let server = TestServer::new_with_regtest_with_index_sats();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/foo", "hello").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{}", InscriptionId { txid, index: 0 }),
      StatusCode::OK,
      r".*<dt>sat</dt>\s*<dd><a href=/sat/5000000000>5000000000</a></dd>\s*<dt>preview</dt>.*",
    );
  }

  #[test]
  fn inscription_page_does_not_have_sat_when_sats_are_not_tracked() {
    let server = TestServer::new_with_regtest();
    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/foo", "hello").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{}", InscriptionId { txid, index: 0 }),
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
      inputs: &[(1, 0, 0, inscription("text/foo", "hello").to_witness())],
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
      inputs: &[(
        1,
        0,
        0,
        Inscription::new(Some("foo/bar".as_bytes().to_vec()), None).to_witness(),
      )],
      ..Default::default()
    });

    let inscription_id = InscriptionId { txid, index: 0 };

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
      inputs: &[(
        1,
        0,
        0,
        Inscription::new(Some("image/png".as_bytes().to_vec()), None).to_witness(),
      )],
      ..Default::default()
    });

    let inscription_id = InscriptionId { txid, index: 0 };

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
      inputs: &[(1, 0, 0, inscription("text/foo", "hello").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    let response = server.get(format!("/content/{}", InscriptionId { txid, index: 0 }));

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
      response.headers().get(header::CACHE_CONTROL).unwrap(),
      "public, max-age=31536000, immutable"
    );
  }

  #[test]
  fn error_content_responses_have_max_age_zero_cache_control_headers() {
    let server = TestServer::new_with_regtest();
    let response =
      server.get("/content/6ac5cacb768794f4fd7a78bf00f2074891fce68bd65c4ff36e77177237aacacai0");

    assert_eq!(response.status(), 404);
    assert_eq!(
      response.headers().get(header::CACHE_CONTROL).unwrap(),
      "no-store"
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
        inputs: &[(i + 1, 0, 0, inscription("text/foo", "hello").to_witness())],
        ..Default::default()
      });
    }

    server.mine_blocks(1);

    server.assert_response_regex(
      "/inscriptions/1",
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
        inputs: &[(i + 1, 0, 0, inscription("text/foo", "hello").to_witness())],
        ..Default::default()
      });
    }

    server.mine_blocks(1);

    server.assert_response_regex(
      "/inscriptions/0",
      StatusCode::OK,
      ".*prev\n<a class=next href=/inscriptions/1>next</a>.*",
    );
  }

  #[test]
  fn collections_page_prev_and_next() {
    let server = TestServer::new_with_regtest_with_index_sats();

    let mut parent_ids = Vec::new();

    for i in 0..101 {
      server.mine_blocks(1);

      parent_ids.push(InscriptionId {
        txid: server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
          inputs: &[(i + 1, 0, 0, inscription("text/plain", "hello").to_witness())],
          ..Default::default()
        }),
        index: 0,
      });
    }

    for (i, parent_id) in parent_ids.iter().enumerate().take(101) {
      server.mine_blocks(1);

      server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[
          (i + 2, 1, 0, Default::default()),
          (
            i + 102,
            0,
            0,
            Inscription {
              content_type: Some("text/plain".into()),
              body: Some("hello".into()),
              parent: Some(parent_id.value()),
              ..Default::default()
            }
            .to_witness(),
          ),
        ],
        outputs: 2,
        output_values: &[50 * COIN_VALUE, 50 * COIN_VALUE],
        ..Default::default()
      });
    }

    server.mine_blocks(1);

    server.assert_response_regex(
      "/collections",
      StatusCode::OK,
      r".*
<h1>Collections</h1>
<div class=thumbnails>
  <a href=/inscription/.*><iframe .* src=/preview/.*></iframe></a>
  (<a href=/inscription/[[:xdigit:]]{64}i0>.*</a>\s*){99}
</div>
<div class=center>
prev
<a class=next href=/collections/1>next</a>
</div>.*"
        .to_string()
        .unindent(),
    );

    server.assert_response_regex(
      "/collections/1",
      StatusCode::OK,
      ".*
<h1>Collections</h1>
<div class=thumbnails>
  <a href=/inscription/.*><iframe .* src=/preview/.*></iframe></a>
</div>
<div class=center>
<a class=prev href=/collections/0>prev</a>
next
</div>.*"
        .unindent(),
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
      .brotli(false)
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
      inputs: &[(
        1,
        0,
        0,
        inscription("text/plain;charset=utf-8", "hello").to_witness(),
      )],
      ..Default::default()
    });
    let inscription = InscriptionId { txid, index: 0 };
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

  #[test]
  fn inscription_links_to_parent() {
    let server = TestServer::new_with_regtest_with_json_api();
    server.mine_blocks(1);

    let parent_txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    let parent_inscription_id = InscriptionId {
      txid: parent_txid,
      index: 0,
    };

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[
        (
          2,
          0,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(parent_inscription_id.value()),
            ..Default::default()
          }
          .to_witness(),
        ),
        (2, 1, 0, Default::default()),
      ],
      ..Default::default()
    });

    server.mine_blocks(1);

    let inscription_id = InscriptionId { txid, index: 0 };

    server.assert_response_regex(
      format!("/inscription/{inscription_id}"),
      StatusCode::OK,
      format!(".*<title>Inscription 1</title>.*<dt>parent</dt>.*<dd><a class=monospace href=/inscription/{parent_inscription_id}>{parent_inscription_id}</a></dd>.*"),
    );

    server.assert_response_regex(
      format!("/inscription/{parent_inscription_id}"),
      StatusCode::OK,
      format!(".*<title>Inscription 0</title>.*<dt>children</dt>.*<a href=/inscription/{inscription_id}>.*</a>.*"),
    );

    assert_eq!(
      server
        .get_json::<InscriptionJson>(format!("/inscription/{inscription_id}"))
        .parent,
      Some(parent_inscription_id),
    );

    assert_eq!(
      server
        .get_json::<InscriptionJson>(format!("/inscription/{parent_inscription_id}"))
        .children,
      [inscription_id],
    );
  }

  #[test]
  fn inscription_with_and_without_children_page() {
    let server = TestServer::new_with_regtest();
    server.mine_blocks(1);

    let parent_txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    let parent_inscription_id = InscriptionId {
      txid: parent_txid,
      index: 0,
    };

    server.assert_response_regex(
      format!("/children/{parent_inscription_id}"),
      StatusCode::OK,
      ".*<h3>No children</h3>.*",
    );

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[
        (
          2,
          0,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(parent_inscription_id.value()),
            ..Default::default()
          }
          .to_witness(),
        ),
        (2, 1, 0, Default::default()),
      ],
      ..Default::default()
    });

    server.mine_blocks(1);

    let inscription_id = InscriptionId { txid, index: 0 };

    server.assert_response_regex(
      format!("/children/{parent_inscription_id}"),
      StatusCode::OK,
      format!(".*<title>Inscription 0 Children</title>.*<h1><a href=/inscription/{parent_inscription_id}>Inscription 0</a> Children</h1>.*<div class=thumbnails>.*<a href=/inscription/{inscription_id}><iframe .* src=/preview/{inscription_id}></iframe></a>.*"),
    );
  }

  #[test]
  fn inscriptions_page_shows_max_four_children() {
    let server = TestServer::new_with_regtest();
    server.mine_blocks(1);

    let parent_txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(6);

    let parent_inscription_id = InscriptionId {
      txid: parent_txid,
      index: 0,
    };

    let _txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[
        (
          2,
          0,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(parent_inscription_id.value()),
            ..Default::default()
          }
          .to_witness(),
        ),
        (
          3,
          0,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(parent_inscription_id.value()),
            ..Default::default()
          }
          .to_witness(),
        ),
        (
          4,
          0,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(parent_inscription_id.value()),
            ..Default::default()
          }
          .to_witness(),
        ),
        (
          5,
          0,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(parent_inscription_id.value()),
            ..Default::default()
          }
          .to_witness(),
        ),
        (
          6,
          0,
          0,
          Inscription {
            content_type: Some("text/plain".into()),
            body: Some("hello".into()),
            parent: Some(parent_inscription_id.value()),
            ..Default::default()
          }
          .to_witness(),
        ),
        (2, 1, 0, Default::default()),
      ],
      ..Default::default()
    });

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{parent_inscription_id}"),
      StatusCode::OK,
      format!(
        ".*<title>Inscription 0</title>.*
.*<a href=/inscription/.*><iframe .* src=/preview/.*></iframe></a>.*
.*<a href=/inscription/.*><iframe .* src=/preview/.*></iframe></a>.*
.*<a href=/inscription/.*><iframe .* src=/preview/.*></iframe></a>.*
.*<a href=/inscription/.*><iframe .* src=/preview/.*></iframe></a>.*
    <div class=center>
      <a href=/children/{parent_inscription_id}>all</a>
    </div>.*"
      ),
    );
  }

  #[test]
  fn inscription_number_endpoint() {
    let server = TestServer::new_with_regtest();
    server.mine_blocks(2);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[
        (1, 0, 0, inscription("text/plain", "hello").to_witness()),
        (2, 0, 0, inscription("text/plain", "cursed").to_witness()),
      ],
      outputs: 2,
      ..Default::default()
    });

    let inscription_id = InscriptionId { txid, index: 0 };
    let cursed_inscription_id = InscriptionId { txid, index: 1 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{inscription_id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription 0</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{inscription_id}</dd>.*"
      ),
    );
    server.assert_response_regex(
      "/inscription/0",
      StatusCode::OK,
      format!(
        ".*<h1>Inscription 0</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{inscription_id}</dd>.*"
      ),
    );

    server.assert_response_regex(
      "/inscription/-1",
      StatusCode::OK,
      format!(
        ".*<h1>Inscription -1</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{cursed_inscription_id}</dd>.*"
      ),
    )
  }

  #[test]
  fn charm_cursed() {
    let server = TestServer::new_with_regtest();

    server.mine_blocks(2);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[
        (1, 0, 0, Witness::default()),
        (2, 0, 0, inscription("text/plain", "cursed").to_witness()),
      ],
      outputs: 2,
      ..Default::default()
    });

    let id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription -1</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{id}</dd>
  <dt>charms</dt>
  <dd>
    <span title=cursed>👹</span>
  </dd>
  .*
</dl>
.*
"
      ),
    );
  }

  #[test]
  fn charm_vindicated() {
    let server = TestServer::new_with_regtest();

    server.mine_blocks(110);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[
        (1, 0, 0, Witness::default()),
        (2, 0, 0, inscription("text/plain", "cursed").to_witness()),
      ],
      outputs: 2,
      ..Default::default()
    });

    let id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription 0</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{id}</dd>
  <dt>charms</dt>
  <dd>
    <span title=vindicated>❤️‍🔥</span>
  </dd>
  .*
</dl>
.*
"
      ),
    );
  }

  #[test]
  fn charm_coin() {
    let server = TestServer::new_with_regtest_with_index_sats();

    server.mine_blocks(2);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "foo").to_witness())],
      ..Default::default()
    });

    let id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription 0</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{id}</dd>
  <dt>charms</dt>
  <dd>.*<span title=coin>🪙</span>.*</dd>
  .*
</dl>
.*
"
      ),
    );
  }

  #[test]
  fn charm_uncommon() {
    let server = TestServer::new_with_regtest_with_index_sats();

    server.mine_blocks(2);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "foo").to_witness())],
      ..Default::default()
    });

    let id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription 0</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{id}</dd>
  <dt>charms</dt>
  <dd>.*<span title=uncommon>🌱</span>.*</dd>
  .*
</dl>
.*
"
      ),
    );
  }

  #[test]
  fn charm_nineball() {
    let server = TestServer::new_with_regtest_with_index_sats();

    server.mine_blocks(9);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(9, 0, 0, inscription("text/plain", "foo").to_witness())],
      ..Default::default()
    });

    let id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription 0</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{id}</dd>
  <dt>charms</dt>
  <dd>.*<span title=nineball>9️⃣</span>.*</dd>
  .*
</dl>
.*
"
      ),
    );
  }

  #[test]
  fn charm_reinscription() {
    let server = TestServer::new_with_regtest();

    server.mine_blocks(1);

    server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "foo").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, inscription("text/plain", "bar").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    let id = InscriptionId { txid, index: 0 };

    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription -1</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{id}</dd>
  <dt>charms</dt>
  <dd>
    <span title=reinscription>♻️</span>
    <span title=cursed>👹</span>
  </dd>
  .*
</dl>
.*
"
      ),
    );
  }

  #[test]
  fn charm_reinscription_in_same_tx_input() {
    let server = TestServer::new_with_regtest();

    server.mine_blocks(1);

    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_slice([1])
      .push_slice(b"text/plain;charset=utf-8")
      .push_slice([])
      .push_slice(b"foo")
      .push_opcode(opcodes::all::OP_ENDIF)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_slice([1])
      .push_slice(b"text/plain;charset=utf-8")
      .push_slice([])
      .push_slice(b"bar")
      .push_opcode(opcodes::all::OP_ENDIF)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_slice([1])
      .push_slice(b"text/plain;charset=utf-8")
      .push_slice([])
      .push_slice(b"qix")
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    let witness = Witness::from_slice(&[script.into_bytes(), Vec::new()]);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, witness)],
      ..Default::default()
    });

    server.mine_blocks(1);

    let id = InscriptionId { txid, index: 0 };
    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription 0</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{id}</dd>
  <dt>output value</dt>
  .*
</dl>
.*
"
      ),
    );

    let id = InscriptionId { txid, index: 1 };
    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      ".*
    <span title=reinscription>♻️</span>
    <span title=cursed>👹</span>.*",
    );

    let id = InscriptionId { txid, index: 2 };
    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      ".*
    <span title=reinscription>♻️</span>
    <span title=cursed>👹</span>.*",
    );
  }

  #[test]
  fn charm_reinscription_in_same_tx_with_pointer() {
    let server = TestServer::new_with_regtest();

    server.mine_blocks(3);

    let cursed_inscription = inscription("text/plain", "bar");
    let reinscription: Inscription = InscriptionTemplate {
      pointer: Some(0),
      ..Default::default()
    }
    .into();

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[
        (1, 0, 0, inscription("text/plain", "foo").to_witness()),
        (2, 0, 0, cursed_inscription.to_witness()),
        (3, 0, 0, reinscription.to_witness()),
      ],
      ..Default::default()
    });

    server.mine_blocks(1);

    let id = InscriptionId { txid, index: 0 };
    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription 0</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{id}</dd>
  <dt>output value</dt>
  .*
</dl>
.*
"
      ),
    );

    let id = InscriptionId { txid, index: 1 };
    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      ".*
    <span title=cursed>👹</span>.*",
    );

    let id = InscriptionId { txid, index: 2 };
    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      ".*
    <span title=reinscription>♻️</span>
    <span title=cursed>👹</span>.*",
    );
  }

  #[test]
  fn charm_unbound() {
    let server = TestServer::new_with_regtest();

    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, envelope(&[b"ord", &[128], &[0]]))],
      ..Default::default()
    });

    server.mine_blocks(1);

    let id = InscriptionId { txid, index: 0 };

    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription -1</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{id}</dd>
  <dt>charms</dt>
  <dd>
    <span title=cursed>👹</span>
    <span title=unbound>🔓</span>
  </dd>
  .*
</dl>
.*
"
      ),
    );
  }

  #[test]
  fn charm_lost() {
    let server = TestServer::new_with_regtest();

    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "foo").to_witness())],
      ..Default::default()
    });

    let id = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription 0</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{id}</dd>
  <dt>output value</dt>
  <dd>5000000000</dd>
  .*
</dl>
.*
"
      ),
    );

    server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Default::default())],
      fee: 50 * COIN_VALUE,
      ..Default::default()
    });

    server.mine_blocks_with_subsidy(1, 0);

    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription 0</h1>.*
<dl>
  <dt>id</dt>
  <dd class=monospace>{id}</dd>
  <dt>charms</dt>
  <dd>
    <span title=lost>🤔</span>
  </dd>
  .*
</dl>
.*
"
      ),
    );
  }

  #[test]
  fn sat_recursive_endpoints() {
    let server = TestServer::new_with_regtest_with_index_sats();

    assert_eq!(
      server.get_json::<SatInscriptionsJson>("/r/sat/5000000000"),
      SatInscriptionsJson {
        ids: vec![],
        page: 0,
        more: false
      }
    );

    assert_eq!(
      server.get_json::<SatInscriptionJson>("/r/sat/5000000000/at/0"),
      SatInscriptionJson { id: None }
    );

    server.mine_blocks(1);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "foo").to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    let mut ids = Vec::new();
    ids.push(InscriptionId { txid, index: 0 });

    for i in 1..111 {
      let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(i + 1, 1, 0, inscription("text/plain", "foo").to_witness())],
        ..Default::default()
      });

      server.mine_blocks(1);

      ids.push(InscriptionId { txid, index: 0 });
    }

    let paginated_response = server.get_json::<SatInscriptionsJson>("/r/sat/5000000000");

    let equivalent_paginated_response =
      server.get_json::<SatInscriptionsJson>("/r/sat/5000000000/0");

    assert_eq!(paginated_response.ids.len(), 100);
    assert!(paginated_response.more);
    assert_eq!(paginated_response.page, 0);

    assert_eq!(
      paginated_response.ids.len(),
      equivalent_paginated_response.ids.len()
    );
    assert_eq!(paginated_response.more, equivalent_paginated_response.more);
    assert_eq!(paginated_response.page, equivalent_paginated_response.page);

    let paginated_response = server.get_json::<SatInscriptionsJson>("/r/sat/5000000000/1");

    assert_eq!(paginated_response.ids.len(), 11);
    assert!(!paginated_response.more);
    assert_eq!(paginated_response.page, 1);

    assert_eq!(
      server
        .get_json::<SatInscriptionJson>("/r/sat/5000000000/at/0")
        .id,
      Some(ids[0])
    );

    assert_eq!(
      server
        .get_json::<SatInscriptionJson>("/r/sat/5000000000/at/-111")
        .id,
      Some(ids[0])
    );

    assert_eq!(
      server
        .get_json::<SatInscriptionJson>("/r/sat/5000000000/at/110")
        .id,
      Some(ids[110])
    );

    assert_eq!(
      server
        .get_json::<SatInscriptionJson>("/r/sat/5000000000/at/-1")
        .id,
      Some(ids[110])
    );

    assert!(server
      .get_json::<SatInscriptionJson>("/r/sat/5000000000/at/111")
      .id
      .is_none());
  }

  #[test]
  fn children_recursive_endpoint() {
    let server = TestServer::new_with_regtest_with_json_api();
    server.mine_blocks(1);

    let parent_txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, inscription("text/plain", "hello").to_witness())],
      ..Default::default()
    });

    let parent_inscription_id = InscriptionId {
      txid: parent_txid,
      index: 0,
    };

    server.assert_response(
      format!("/r/children/{parent_inscription_id}"),
      StatusCode::NOT_FOUND,
      &format!("inscription {parent_inscription_id} not found"),
    );

    server.mine_blocks(1);

    let children_json =
      server.get_json::<ChildrenJson>(format!("/r/children/{parent_inscription_id}"));
    assert_eq!(children_json.ids.len(), 0);

    let mut builder = script::Builder::new();
    for _ in 0..111 {
      builder = Inscription {
        content_type: Some("text/plain".into()),
        body: Some("hello".into()),
        parent: Some(parent_inscription_id.value()),
        unrecognized_even_field: false,
        ..Default::default()
      }
      .append_reveal_script_to_builder(builder);
    }

    let witness = Witness::from_slice(&[builder.into_bytes(), Vec::new()]);

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 0, 0, witness), (2, 1, 0, Default::default())],
      ..Default::default()
    });

    server.mine_blocks(1);

    let first_child_inscription_id = InscriptionId { txid, index: 0 };
    let hundredth_child_inscription_id = InscriptionId { txid, index: 99 };
    let hundred_first_child_inscription_id = InscriptionId { txid, index: 100 };
    let hundred_eleventh_child_inscription_id = InscriptionId { txid, index: 110 };

    let children_json =
      server.get_json::<ChildrenJson>(format!("/r/children/{parent_inscription_id}"));

    assert_eq!(children_json.ids.len(), 100);
    assert_eq!(children_json.ids[0], first_child_inscription_id);
    assert_eq!(children_json.ids[99], hundredth_child_inscription_id);
    assert!(children_json.more);
    assert_eq!(children_json.page, 0);

    let children_json =
      server.get_json::<ChildrenJson>(format!("/r/children/{parent_inscription_id}/1"));

    assert_eq!(children_json.ids.len(), 11);
    assert_eq!(children_json.ids[0], hundred_first_child_inscription_id);
    assert_eq!(children_json.ids[10], hundred_eleventh_child_inscription_id);
    assert!(!children_json.more);
    assert_eq!(children_json.page, 1);
  }

  #[test]
  fn inscriptions_in_block_page() {
    let server = TestServer::new_with_regtest_with_index_sats();

    for _ in 0..101 {
      server.mine_blocks(1);
    }

    for i in 0..101 {
      server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(i + 1, 0, 0, inscription("text/foo", "hello").to_witness())],
        ..Default::default()
      });
    }

    server.mine_blocks(1);

    server.assert_response_regex(
      "/inscriptions/block/102",
      StatusCode::OK,
      r".*(<a href=/inscription/[[:xdigit:]]{64}i0>.*</a>.*){100}.*",
    );

    server.assert_response_regex(
      "/inscriptions/block/102/1",
      StatusCode::OK,
      r".*<a href=/inscription/[[:xdigit:]]{64}i0>.*</a>.*",
    );
  }

  #[test]
  fn inscription_query_display() {
    assert_eq!(
      InscriptionQuery::Id(inscription_id(1)).to_string(),
      "1111111111111111111111111111111111111111111111111111111111111111i1"
    );
    assert_eq!(InscriptionQuery::Number(1).to_string(), "1")
  }

  #[test]
  fn inscription_not_found() {
    TestServer::new_with_regtest_with_json_api().assert_response(
      "/inscription/0",
      StatusCode::NOT_FOUND,
      "inscription 0 not found",
    );
  }

  #[test]
  fn delegate() {
    let server = TestServer::new_with_regtest();

    server.mine_blocks(1);

    let delegate = Inscription {
      content_type: Some("text/html".into()),
      body: Some("foo".into()),
      ..Default::default()
    };

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, delegate.to_witness())],
      ..Default::default()
    });

    let delegate = InscriptionId { txid, index: 0 };

    server.mine_blocks(1);

    let inscription = Inscription {
      delegate: Some(delegate.value()),
      ..Default::default()
    };

    let txid = server.bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 0, 0, inscription.to_witness())],
      ..Default::default()
    });

    server.mine_blocks(1);

    let id = InscriptionId { txid, index: 0 };

    server.assert_response_regex(
      format!("/inscription/{id}"),
      StatusCode::OK,
      format!(
        ".*<h1>Inscription 1</h1>.*
        <dl>
          <dt>id</dt>
          <dd class=monospace>{id}</dd>
          .*
          <dt>delegate</dt>
          <dd><a href=/inscription/{delegate}>{delegate}</a></dd>
          .*
        </dl>.*"
      )
      .unindent(),
    );

    server.assert_response(format!("/content/{id}"), StatusCode::OK, "foo");

    server.assert_response(format!("/preview/{id}"), StatusCode::OK, "foo");
  }
}
