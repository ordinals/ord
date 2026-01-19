use super::*;

#[derive(Boilerplate)]
pub(crate) struct GalleriesHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) prev: Option<u32>,
  pub(crate) next: Option<u32>,
}

impl PageContent for GalleriesHtml {
  fn title(&self) -> String {
    "Galleries".into()
  }
}
