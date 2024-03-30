use super::*;

#[derive(Debug, PartialEq)]
pub enum Artifact {
  Cenotaph(Cenotaph),
  Runestone(Runestone),
}
