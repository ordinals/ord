use {
  super::*,
  crate::subcommand::server::query,
  tantivy::{
    collector::{Count, TopDocs},
    directory::MmapDirectory,
    query::QueryParser,
    schema::{document::OwnedValue, Field, Schema as TantivySchema, STORED, STRING},
    Index as TantivyIndex, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument,
  },
};

#[derive(Clone)]
struct Schema {
  inscription_id: Field,
  sat_name: Field,
}

impl Schema {
  fn default_search_fields(&self) -> Vec<Field> {
    vec![self.inscription_id, self.sat_name]
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

  fn query_parser(&self, search_index: &TantivyIndex) -> QueryParser {
    let mut query_parser = QueryParser::for_index(search_index, self.default_search_fields());
    query_parser.set_conjunction_by_default();
    query_parser
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

pub struct SearchResult {
  pub inscription_id: InscriptionId,
}

impl SearchIndex {
  pub fn open(index: Arc<Index>, settings: &Settings) -> Result<Self> {
    let mut schema_builder = TantivySchema::builder();

    let document = Schema {
      inscription_id: schema_builder.add_text_field("inscription_id", STRING | STORED),
      sat_name: schema_builder.add_text_field("sat_name", STRING),
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
      schema: document,
      search_index,
      writer: Arc::new(Mutex::new(writer)),
    })
  }

  pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
    let searcher = self.reader.searcher();

    let query = self
      .schema
      .query_parser(&self.search_index)
      .parse_query(query)?;

    Ok(
      searcher
        .search(&query, &TopDocs::with_limit(100))?
        .iter()
        .filter_map(|(_score, doc_address)| {
          self
            .schema
            .search_result(&searcher.doc::<TantivyDocument>(*doc_address).ok()?)
        })
        .collect(),
    )
  }

  pub(crate) fn update(&self) -> Result {
    let mut indexed_inscriptions = Vec::new();

    loop {
      for inscription_id in self.ord_index.get_all_inscriptions()? {
        if !indexed_inscriptions.contains(&inscription_id) {
          self.index_inscription(inscription_id)?;
          indexed_inscriptions.push(inscription_id);
        }

        if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
          return Ok(())
        }
      }
    }
  }

  fn index_inscription(&self, inscription_id: InscriptionId) -> Result {
    let searcher = self.reader.searcher();

    let query = self
      .schema
      .query_parser(&self.search_index)
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

    if let Some(sat) = inscription.sat {
      document.add_text(self.schema.sat_name, sat.name());
    }

    writer.add_document(document)?;

    writer.commit()?;

    log::info!(
      "Added inscription with id `{}` to search index",
      inscription_id
    );

    Ok(())
  }
}
