use super::*;

#[derive(Boilerplate)]
pub(crate) struct RunesHtml {
  pub(crate) entries: Vec<(u64, RuneEntry)>,
}

impl PageContent for RunesHtml {
  fn title(&self) -> String {
    "Runes".to_string()
  }
}
