use super::*;

#[derive(Boilerplate)]
pub(crate) struct RunesHtml {
  pub(crate) entries: Vec<(u64, RuneEntry)>,
}
