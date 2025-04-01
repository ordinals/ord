use {super::*, search_index::SearchResult};

#[derive(Boilerplate)]
pub(crate) struct ExploreHtml {
  pub(crate) search_results: Vec<SearchResult>,
}

impl PageContent for ExploreHtml {
  fn title(&self) -> String {
    "Explore".to_string()
  }
}
