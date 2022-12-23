use super::*;

pub(crate) struct ContentHtml<'a> {
  pub(crate) content: Option<Content<'a>>,
  pub(crate) inscription_id: InscriptionId,
}

impl<'a> Display for ContentHtml<'a> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.content {
      Some(Content::Text(text)) => {
        write!(f, "<pre>")?;
        text.escape(f, false)?;
        write!(f, "</pre>")
      }
      Some(Content::Image) => write!(f, "<img src=/content/{}>", self.inscription_id),
      Some(Content::IFrame) => {
        write!(f, "<iframe src=/content/{}></iframe>", self.inscription_id)
      }
      None => write!(f, "<p>UNKNOWN</p>"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn text() {
    assert_eq!(
      ContentHtml {
        content: Some(Content::Text("foo")),
        inscription_id: txid(1),
      }
      .to_string(),
      "<pre>foo</pre>"
    );
  }

  #[test]
  fn image() {
    assert_eq!(
      ContentHtml {
        content: Some(Content::Image),
        inscription_id: txid(1),
      }
      .to_string(),
      "<img src=/content/1111111111111111111111111111111111111111111111111111111111111111>"
    );
  }

  #[test]
  fn iframe() {
    assert_eq!(
      ContentHtml {
        content: Some(Content::IFrame),
        inscription_id: txid(1),
      }
      .to_string(),
      "<iframe src=/content/1111111111111111111111111111111111111111111111111111111111111111></iframe>"
    );
  }
}
