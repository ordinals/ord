use {
  super::*,
  crate::index::event::Event,
  tantivy::{
    schema::{Field, Schema, STORED, STRING, TEXT},
    Index as TantivyIndex, IndexReader, IndexWriter, ReloadPolicy,
  },
  tokio::sync::{mpsc, RwLock},
};

#[derive(Clone)]
pub struct SearchIndex {
  fields: Fields,
  ord_index: Arc<Index>,
  reader: IndexReader,
  schema: Schema,
  search_index: TantivyIndex,
  writer: Arc<RwLock<IndexWriter>>,
}

#[derive(Clone)]
struct Fields {
  id: Field,
  content: Field,
  content_type: Field, 
  sat_name: Field,
}

impl SearchIndex {
  pub fn open(settings: &Settings) -> Result<Self> {
    let mut schema_builder = Schema::builder();

    let fields = Fields {
      id: schema_builder.add_text_field("id", STORED | STRING),
      content: schema_builder.add_text_field("content", STORED | TEXT),
      content_type: schema_builder.add_text_field("content_type", STORED | TEXT),
      sat_name: schema_builder.add_text_field("sat_name", STORED | STRING),
    };

    let schema = schema_builder.build();

    let search_index = match TantivyIndex::open_in_dir("ord_search_index") {
      Ok(index) => index,
      Err(_) => TantivyIndex::create_in_dir("ord_search_index", schema.clone())?,
    };

    let reader = search_index
      .reader_builder()
      .reload_policy(ReloadPolicy::OnCommitWithDelay)
      .try_into()?;

    let writer = search_index.writer(50_000_000)?;

    let (event_sender, mut event_receiver) = mpsc::channel(1024);

    let ord_index = Arc::new(Index::open_with_event_sender(
      &settings,
      Some(event_sender),
    )?);

    let search_index = Self {
      fields,
      ord_index: ord_index.clone(),
      reader,
      schema,
      search_index,
      writer: Arc::new(RwLock::new(writer)),
    };

    let search_index_clone = search_index.clone();

    thread::spawn(move || {
      while let Some(event) = event_receiver.blocking_recv() {
        match event {
          Event::InscriptionCreated { inscription_id, .. } => {
            if let Err(e) = search_index_clone.add_inscription(inscription_id) {
              eprintln!("Failed to index inscription {}: {}", inscription_id, e);
            }
          }
          _ => {}
        }
      }
    });

    ord_index.update()?;

    Ok(search_index)
  }

  pub fn search(&self, query: &str) -> Result {
    Ok(())
  }

  fn add_inscription(&self, inscription_id: InscriptionId) -> Result {
    Ok(())
  }
}
