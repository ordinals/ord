use super::*;

#[derive(Debug, PartialEq)]
pub enum Error {
  Script(script::Error),
  Varint,
}

impl From<script::Error> for Error {
  fn from(error: script::Error) -> Self {
    Self::Script(error)
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Script(err) => write!(f, "failed to parse script: {err}"),
      Self::Varint => write!(f, "varint over maximum value"),
    }
  }
}

impl std::error::Error for Error {}
