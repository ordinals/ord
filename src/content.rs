pub(crate) enum Content<'a> {
  Text(&'a str),
  Png(&'a [u8]),
}
