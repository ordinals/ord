#[derive(Debug, PartialEq)]
pub(crate) enum Content<'a> {
  Image,
  Svg,
  Text(&'a str),
}
