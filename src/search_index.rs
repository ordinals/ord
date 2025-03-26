use {
  super::*,
  crate::index::event::Event,
  crate::subcommand::server::query,
  tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{document::OwnedValue, Field, Schema as TantivySchema, STORED, STRING},
    Index as TantivyIndex, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument,
  },
  tokio::sync::mpsc::Receiver,
};

#[derive(Clone)]
struct Schema {
  id: Field,
  sat_name: Field,
}

impl Schema {
  fn to_search_result(&self, result: &TantivyDocument) -> Option<SearchResult> {
    let id_str = result.get_first(self.id).and_then(|value| {
      if let OwnedValue::Str(id_str) = value {
        Some(id_str)
      } else {
        None
      }
    })?;

    let inscription_id = id_str.parse::<InscriptionId>().ok()?;

    let sat_name = result.get_first(self.sat_name).and_then(|value| {
      if let OwnedValue::Str(name) = value {
        Some(name.clone())
      } else {
        None
      }
    });

    Some(SearchResult {
      inscription_id,
      sat_name,
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

#[allow(unused)]
pub struct SearchResult {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) sat_name: Option<String>,
}

impl SearchIndex {
  pub fn open(index: Arc<Index>, event_receiver: Receiver<Event>) -> Result<Self> {
    let mut event_receiver = event_receiver;

    let mut schema_builder = TantivySchema::builder();

    let document = Schema {
      id: schema_builder.add_text_field("id", STORED | STRING),
      sat_name: schema_builder.add_text_field("sat_name", STORED | STRING),
    };

    let schema = schema_builder.build();

    // TODO: don't hard code this
    let path = PathBuf::from("ord_search_index");

    if !path.exists() {
      fs::create_dir(&path)?;
    }

    let search_index = match TantivyIndex::open_in_dir(&path) {
      Ok(index) => index,
      Err(_) => TantivyIndex::create_in_dir(&path, schema)?,
    };

    let reader = search_index
      .reader_builder()
      .reload_policy(ReloadPolicy::OnCommitWithDelay)
      .try_into()?;

    let writer = search_index.writer(50_000_000)?;

    let search_index = Self {
      ord_index: index,
      reader,
      schema: document,
      search_index,
      writer: Arc::new(Mutex::new(writer)),
    };

    let search_index_clone = search_index.clone();

    thread::spawn(move || {
      while let Some(event) = event_receiver.blocking_recv() {
        if let Event::InscriptionCreated { inscription_id, .. } = event {
          if let Err(error) = search_index_clone.add_inscription(inscription_id) {
            log::error!(
              "failed to add inscription with id `{}` to search index: {}",
              inscription_id,
              error
            );
          }
        }
      }
    });

    Ok(search_index)
  }

  pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
    let searcher = self.reader.searcher();

    let mut query_parser = QueryParser::for_index(&self.search_index, vec![self.schema.sat_name]);

    query_parser.set_conjunction_by_default();

    let query = query_parser.parse_query(query)?;

    Ok(
      searcher
        .search(&query, &TopDocs::with_limit(100))?
        .iter()
        .filter_map(|(_score, doc_address)| {
          self
            .schema
            .to_search_result(&searcher.doc::<TantivyDocument>(*doc_address).ok()?)
        })
        .collect(),
    )
  }

  fn add_inscription(&self, inscription_id: InscriptionId) -> Result {
    let inscription_info = self
      .ord_index
      .inscription_info(query::Inscription::Id(inscription_id), None)?
      .ok_or(anyhow!(format!(
        "failed to get info for inscription with id `{inscription_id}`"
      )))?;

    let (inscription, _, _) = inscription_info;

    let mut writer = self.writer.lock().unwrap();

    let mut document = TantivyDocument::default();

    document.add_text(self.schema.id, inscription.id.to_string());

    if let Some(sat) = inscription.sat {
      document.add_text(self.schema.sat_name, sat.name());
    }

    writer.add_document(document)?;

    writer.commit()?;

    Ok(())
  }
}
