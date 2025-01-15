use super::*;

#[derive(Boilerplate)]
pub(crate) struct SatscardHtml {
  pub(crate) query: Option<crate::satscard::Query>,
}

impl PageContent for SatscardHtml {
  fn title(&self) -> String {
    // inlude address, slot, and state?
    "SATSCARD".into()
  }
}
