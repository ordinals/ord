use super::*;

pub(crate) struct ContentHtml<'a> {
  pub(crate) content: Option<Content<'a>>,
  pub(crate) inscription_id: InscriptionId,
}

impl<'a> Display for ContentHtml<'a> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.content {
      Some(Content::Text(text)) => {
        write!(f, "<p>")?;
        text.escape(f, false)?;
        write!(f, "</p>")
      }
      Some(Content::Image) => write!(f, "<img src=/content/{}>", self.inscription_id),
      None => write!(f, "<p>UNKNOWN</p>"),
    }
  }
}
