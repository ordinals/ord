#[derive(Debug, PartialEq)]
pub(crate) enum Content<'a> {
  Text(&'a str),
  Image,
}
