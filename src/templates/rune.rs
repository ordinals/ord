use super::*;

#[derive(Boilerplate)]
pub(crate) struct RuneHtml {
  pub(crate) entry: RuneEntry,
}

impl PageContent for RuneHtml {
  fn title(&self) -> String {
    format!("Rune {}", self.entry.rune)
  }
}
