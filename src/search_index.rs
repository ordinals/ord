use {
  super::*,
  crate::index::event::Event,
  crate::subcommand::server::query,
  tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    query::QueryParser,
    schema::{document::OwnedValue, Field, Schema as TantivySchema, STORED, STRING},
    Index as TantivyIndex, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument,
  },
  tokio::sync::mpsc::Receiver,
};

pub struct Config<'a> {
  pub(crate) event_receiver: Receiver<Event>,
  pub(crate) index: Arc<Index>,
  pub(crate) settings: &'a Settings,
}

#[derive(Debug, Clone, Copy)]
enum PendingInscriptionAction {
  Create,
  Update,
}

#[derive(Debug, Clone)]
struct PendingInscription {
  inscription_id: InscriptionId,
  action: PendingInscriptionAction,
}

#[derive(Clone)]
struct Schema {
  inscription_id: Field,
  sat_name: Field,
}

impl Schema {
  fn search_fields(&self) -> Vec<Field> {
    vec![self.inscription_id, self.sat_name]
  }

  fn result(&self, document: &TantivyDocument) -> Option<SearchResult> {
    let id_str = document.get_first(self.inscription_id).and_then(|value| {
      if let OwnedValue::Str(id_str) = value {
        Some(id_str)
      } else {
        None
      }
    })?;

    let inscription_id = id_str.parse::<InscriptionId>().ok()?;

    let sat_name = document.get_first(self.sat_name).and_then(|value| {
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
  pub fn open(config: Config<'_>) -> Result<Self> {
    let mut event_receiver = config.event_receiver;

    let mut schema_builder = TantivySchema::builder();

    let document = Schema {
      inscription_id: schema_builder.add_text_field("inscription_id", STORED | STRING),
      sat_name: schema_builder.add_text_field("sat_name", STORED | STRING),
    };

    let path = config.settings.search_index().to_owned();

    fs::create_dir_all(&path).snafu_context(error::Io { path: path.clone() })?;

    let search_index =
      TantivyIndex::open_or_create(MmapDirectory::open(path)?, schema_builder.build())?;

    let reader = search_index
      .reader_builder()
      .reload_policy(ReloadPolicy::OnCommitWithDelay)
      .try_into()?;

    let writer = search_index.writer(50_000_000)?;

    let search_index = Self {
      ord_index: config.index,
      reader,
      schema: document,
      search_index,
      writer: Arc::new(Mutex::new(writer)),
    };

    let search_index_clone = search_index.clone();

    let inscription_ids = search_index.ord_index.get_all_inscriptions()?;

    thread::spawn(move || {
      for inscription_id in inscription_ids {
        if let Err(error) = search_index_clone.add_inscription(inscription_id) {
          log::error!(
            "failed to add inscription with id `{}` to search index: {}",
            inscription_id,
            error
          );
        }
      }
    });

    let search_index_clone = search_index.clone();

    let pending_inscriptions = Arc::new(Mutex::new(Vec::new()));
    let pending_clone = pending_inscriptions.clone();

    thread::spawn(move || {
      while let Some(event) = event_receiver.blocking_recv() {
        match event {
          Event::InscriptionCreated { inscription_id, .. } => {
            pending_clone.lock().unwrap().push(PendingInscription {
              inscription_id,
              action: PendingInscriptionAction::Create,
            });
          }
          Event::InscriptionTransferred { inscription_id, .. } => {
            pending_clone.lock().unwrap().push(PendingInscription {
              inscription_id,
              action: PendingInscriptionAction::Update,
            });
          }
          Event::Commit => {
            let mut pending = pending_clone.lock().unwrap();

            for pending_inscription in pending.drain(..) {
              match pending_inscription.action {
                PendingInscriptionAction::Update => {
                  if let Err(error) =
                    search_index_clone.update_inscription(pending_inscription.inscription_id)
                  {
                    log::error!(
                      "failed to update inscription with id `{}` to search index: {}",
                      pending_inscription.inscription_id,
                      error
                    );
                  }
                }
                PendingInscriptionAction::Create => {
                  if let Err(error) =
                    search_index_clone.add_inscription(pending_inscription.inscription_id)
                  {
                    log::error!(
                      "failed to add inscription with id `{}` to search index: {}",
                      pending_inscription.inscription_id,
                      error
                    );
                  }
                }
              }
            }
          }
          _ => {}
        }
      }
    });

    Ok(search_index)
  }

  pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
    let searcher = self.reader.searcher();

    let mut query_parser = QueryParser::for_index(&self.search_index, self.schema.search_fields());

    query_parser.set_conjunction_by_default();
    query_parser.set_field_fuzzy(self.schema.inscription_id, true, 2, true);
    query_parser.set_field_fuzzy(self.schema.sat_name, true, 2, true);

    let query = query_parser.parse_query(query)?;

    Ok(
      searcher
        .search(&query, &TopDocs::with_limit(100))?
        .iter()
        .filter_map(|(_score, doc_address)| {
          self
            .schema
            .result(&searcher.doc::<TantivyDocument>(*doc_address).ok()?)
        })
        .collect(),
    )
  }

  fn add_inscription(&self, inscription_id: InscriptionId) -> Result {
    let searcher = self.reader.searcher();

    let query_parser = QueryParser::for_index(&self.search_index, vec![self.schema.inscription_id]);

    let query = query_parser.parse_query(&format!("\"{}\"", inscription_id))?;

    if !searcher.search(&query, &TopDocs::with_limit(1))?.is_empty() {
      log::info!(
        "Inscription with id `{}` already exists in search index, skipping",
        inscription_id
      );

      return Ok(());
    }

    let inscription_info = self
      .ord_index
      .inscription_info(query::Inscription::Id(inscription_id), None)?
      .ok_or(anyhow!(format!(
        "failed to get info for inscription with id `{inscription_id}`"
      )))?;

    let (inscription, _, _) = inscription_info;

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

  fn update_inscription(&self, inscription_id: InscriptionId) -> Result {
    let inscription_info = self
      .ord_index
      .inscription_info(query::Inscription::Id(inscription_id), None)?
      .ok_or(anyhow!(format!(
        "failed to get info for inscription with id `{inscription_id}`"
      )))?;

    let (inscription, _, _) = inscription_info;

    let mut writer = self.writer.lock().unwrap();

    writer.delete_term(tantivy::Term::from_field_text(
      self.schema.inscription_id,
      &inscription_id.to_string(),
    ));

    let mut document = TantivyDocument::default();

    document.add_text(self.schema.inscription_id, inscription.id.to_string());

    if let Some(sat) = inscription.sat {
      document.add_text(self.schema.sat_name, sat.name());
    }

    writer.add_document(document)?;

    writer.commit()?;

    log::info!(
      "Updated inscription with id `{}` in search index",
      inscription_id
    );

    Ok(())
  }
}
