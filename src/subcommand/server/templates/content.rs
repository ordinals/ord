use super::*;

pub(crate) struct ContentHtml<'a>(pub(crate) Option<Content<'a>>);

impl<'a> Display for ContentHtml<'a> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.0 {
      Some(Content::Text(text)) => text.escape(f, false),
      Some(Content::Png(png)) => write!(
        f,
        "<img src='data:image/png;base64,{}'>",
        base64::encode(png)
      ),
      None => write!(f, "UNKNOWN"),
    }
  }
}
