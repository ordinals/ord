use super::*;

#[derive(Boilerplate)]
pub(crate) struct SatscardHtml {
  pub(crate) satscard: Option<Satscard>,
}

impl PageContent for SatscardHtml {
  fn title(&self) -> String {
    // todo: inlude address, slot, and state?
    "SATSCARD".into()
  }
}
