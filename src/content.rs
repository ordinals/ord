#[derive(Debug, PartialEq)]
pub(crate) enum Content<'a> {
  Html,
  Image,
  Svg,
  Text(&'a str),
}
