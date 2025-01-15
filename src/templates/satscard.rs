use super::*;

#[derive(Boilerplate)]
pub(crate) struct SatscardHtml {
  pub(crate) address_info: Option<AddressHtml>,
  pub(crate) satscard: Option<Satscard>,
}

impl SatscardHtml {
  fn form_value(&self) -> Option<String> {
    self
      .satscard
      .as_ref()
      .map(|satscard| format!("https://getsatscard.com/start{}#", satscard.parameters))
  }
}

impl PageContent for SatscardHtml {
  fn title(&self) -> String {
    // todo: inlude address, slot, and state?
    "SATSCARD".into()
  }
}
