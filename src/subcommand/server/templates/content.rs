use super::*;

pub(crate) struct ContentHtml<'a>(pub(crate) Option<Content<'a>>);

impl<'a> Display for ContentHtml<'a> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.0 {
      Some(Content::Text(text)) => {
        write!(f, "<p>")?;
        text.escape(f, false)?;
        write!(f, "</p>")
      }
      Some(Content::Png(png)) => write!(
        f,
        "<img src='data:image/png;base64,{}'>",
        base64::encode(png)
      ),
      None => write!(f, "<p>UNKNOWN</p>"),
    }
  }
}
