use super::*;

pub(crate) struct Accept(pub(crate) HeaderValue);

impl Header for Accept {
  fn name() -> &'static HeaderName {
    &http::header::ACCEPT
  }

  fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
  where
    I: Iterator<Item = &'i HeaderValue>,
  {
    values
      .next()
      .cloned()
      .map(Self)
      .ok_or_else(headers::Error::invalid)
  }

  fn encode<E>(&self, values: &mut E)
  where
    E: Extend<HeaderValue>,
  {
    values.extend(std::iter::once(self.0.clone()));
  }
}
