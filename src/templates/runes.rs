use super::*;

#[derive(Boilerplate)]
pub(crate) struct RunesHtml {
  pub(crate) etchings: Vec<(u64, Etching)>,
}
