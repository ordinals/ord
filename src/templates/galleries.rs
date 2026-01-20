use super::*;

#[derive(Boilerplate)]
pub(crate) struct GalleriesHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) prev: Option<usize>,
  pub(crate) next: Option<usize>,
}

impl PageContent for GalleriesHtml {
  fn title(&self) -> String {
    "Galleries".into()
  }
}
