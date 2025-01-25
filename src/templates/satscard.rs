use super::*;

#[derive(Boilerplate)]
pub(crate) struct SatscardHtml {
  pub(crate) satscard: Option<(Satscard, Option<AddressHtml>)>,
}

impl SatscardHtml {
  fn form_value(&self) -> Option<String> {
    self.satscard.as_ref().map(|(satscard, _address_info)| {
      format!(
        "https://getsatscard.com/start{}#",
        satscard.query_parameters
      )
    })
  }
}

impl PageContent for SatscardHtml {
  fn title(&self) -> String {
    // todo: inlude address, slot, and state?
    "SATSCARD".into()
  }
}
