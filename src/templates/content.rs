use super::*;

pub(crate) struct ContentHtml<'a> {
  pub(crate) content: Option<Content<'a>>,
  pub(crate) inscription_id: InscriptionId,
}

impl<'a> Display for ContentHtml<'a> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.content {
      Some(Content::Text(text)) => {
        write!(f, "<pre class=inscription>")?;
        text.escape(f, false)?;
        write!(f, "</pre>")
      }
      Some(Content::Image) => write!(
        f,
        "<img class=inscription src=/content/{}>",
        self.inscription_id
      ),
      Some(Content::IFrame) => {
        write!(
          f,
          "<iframe class=inscription sandbox=allow-scripts src=/content/{}></iframe>",
          self.inscription_id
        )
      }
      None => write!(f, "<p>UNKNOWN</p>"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unknown() {
    assert_eq!(
      ContentHtml {
        content: None,
        inscription_id: txid(1),
      }
      .to_string(),
      "<p>UNKNOWN</p>"
    );
  }

  #[test]
  fn text() {
    assert_eq!(
      ContentHtml {
        content: Some(Content::Text("foo")),
        inscription_id: txid(1),
      }
      .to_string(),
      "<pre class=inscription>foo</pre>"
    );
  }

  #[test]
  fn text_is_escaped() {
    assert_eq!(
      ContentHtml {
        content: Some(Content::Text("<script>alert('hello!')</script>")),
        inscription_id: txid(1),
      }
      .to_string(),
      "<pre class=inscription>&lt;script&gt;alert(&apos;hello!&apos;)&lt;/script&gt;</pre>",
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
      "<img class=inscription src=/content/1111111111111111111111111111111111111111111111111111111111111111>"
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
      "<iframe class=inscription sandbox=allow-scripts src=/content/1111111111111111111111111111111111111111111111111111111111111111></iframe>"
    );
  }
}
