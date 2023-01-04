#[derive(Debug, PartialEq)]
pub(crate) enum Content<'a> {
  Iframe,
  Image,
  Text(&'a str),
}
