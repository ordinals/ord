use {
  super::*,
  crate::subcommand::server::query,
  tantivy::{
    collector::{Count, TopDocs},
    directory::MmapDirectory,
    query::QueryParser,
    schema::{
      document::OwnedValue, DateOptions, DateTimePrecision, Field, Schema as TantivySchema,
      INDEXED, STORED, STRING,
    },
    DateTime, Index as TantivyIndex, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument,
  },
};

#[derive(Clone)]
struct Schema {
  inscription_id: Field,
  charm: Field,
  sat_name: Field,
  timestamp: Field,
}

impl Schema {
  fn default_search_fields(&self) -> Vec<Field> {
    vec![
      self.inscription_id,
      self.charm,
      self.sat_name,
      self.timestamp,
    ]
  }

  fn search_result(&self, document: &TantivyDocument) -> Option<SearchResult> {
    let inscription_id = document.get_first(self.inscription_id).and_then(|value| {
      if let OwnedValue::Str(id_str) = value {
        Some(id_str)
      } else {
        None
      }
    })?;

    Some(SearchResult {
      inscription_id: inscription_id.parse().ok()?,
    })
  }
}

#[derive(Clone)]
pub struct SearchIndex {
  ord_index: Arc<Index>,
  reader: IndexReader,
  schema: Schema,
  search_index: TantivyIndex,
  writer: Arc<Mutex<IndexWriter>>,
}

#[derive(Eq, Hash, PartialEq)]
pub struct SearchResult {
  pub inscription_id: InscriptionId,
}

impl SearchIndex {
  pub fn open(index: Arc<Index>, settings: &Settings) -> Result<Self> {
    let mut schema_builder = TantivySchema::builder();

    let schema = Schema {
      inscription_id: schema_builder.add_text_field("inscription_id", STRING | STORED),
      charm: schema_builder.add_text_field("charm", STRING),
      sat_name: schema_builder.add_text_field("sat_name", STRING),
      timestamp: schema_builder.add_date_field(
        "timestamp",
        DateOptions::from(INDEXED)
          .set_fast()
          .set_precision(DateTimePrecision::Seconds),
      ),
    };

    let path = settings.search_index().to_owned();

    fs::create_dir_all(&path).snafu_context(error::Io { path: path.clone() })?;

    let search_index =
      TantivyIndex::open_or_create(MmapDirectory::open(path)?, schema_builder.build())?;

    let reader = search_index
      .reader_builder()
      .reload_policy(ReloadPolicy::OnCommitWithDelay)
      .try_into()?;

    let writer = search_index.writer(50_000_000)?;

    Ok(Self {
      ord_index: index,
      reader,
      schema,
      search_index,
      writer: Arc::new(Mutex::new(writer)),
    })
  }

  pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
    let searcher = self.reader.searcher();

    let query = self.query_parser().parse_query(query)?;

    Ok(searcher
      .search(&query, &TopDocs::with_limit(100))?
      .iter()
      .filter_map(|(_score, doc_address)| {
        self
          .schema
          .search_result(&searcher.doc::<TantivyDocument>(*doc_address).ok()?)
      })
      .collect())
  }

  pub fn update(&self) -> Result {
    let mut indexed_inscriptions = Vec::new();

    loop {
      for inscription_id in self.ord_index.get_inscriptions()? {
        if !indexed_inscriptions.contains(&inscription_id) {
          self.index_inscription(inscription_id)?;
          indexed_inscriptions.push(inscription_id);
        }

        if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
          return Ok(());
        }
      }
    }
  }

  fn index_inscription(&self, inscription_id: InscriptionId) -> Result {
    let searcher = self.reader.searcher();

    let query = self
      .query_parser()
      .parse_query(&format!("inscription_id:{inscription_id}"))?;

    if searcher.search(&query, &Count)? > 0 {
      return Ok(());
    }

    let (inscription, _, _) = self
      .ord_index
      .inscription_info(query::Inscription::Id(inscription_id), None)?
      .ok_or(anyhow!(format!(
        "failed to get info for inscription with id `{inscription_id}`"
      )))?;

    let mut writer = self.writer.lock().unwrap();

    let mut document = TantivyDocument::default();

    document.add_text(self.schema.inscription_id, inscription.id.to_string());

    for charm in inscription.charms {
      document.add_text(self.schema.charm, charm);
    }

    if let Some(sat) = inscription.sat {
      document.add_text(self.schema.sat_name, sat.name());
    }

    document.add_date(
      self.schema.timestamp,
      DateTime::from_timestamp_secs(inscription.timestamp),
    );

    writer.add_document(document)?;

    writer.commit()?;

    log::info!(
      "Indexed inscription with id `{}` to search index",
      inscription_id
    );

    Ok(())
  }

  fn query_parser(&self) -> QueryParser {
    let mut query_parser =
      QueryParser::for_index(&self.search_index, self.schema.default_search_fields());

    query_parser.set_conjunction_by_default();

    query_parser
  }
}
