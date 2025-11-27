use super::*;

#[derive(Boilerplate)]
pub(crate) struct AddressHtml {
  pub(crate) address: Address,
  pub(crate) header: bool,
  pub(crate) inscriptions: Option<Vec<InscriptionId>>,
  pub(crate) outputs: Vec<OutPoint>,
  pub(crate) sat_balance: u64,
}

impl PageContent for AddressHtml {
  fn title(&self) -> String {
    format!("Address {}", self.address)
  }
}