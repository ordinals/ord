#[derive(Debug, PartialEq)]
pub(crate) enum Content<'a> {
  IFrame,
  Image,
  Text(&'a str),
}
