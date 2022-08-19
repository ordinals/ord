use super::*;

#[derive(Debug)]
pub(crate) enum Expected {
  String(String),
  Regex(Regex),
  Ignore,
}

impl Expected {
  pub(crate) fn regex(pattern: &str) -> Self {
    Self::Regex(Regex::new(&format!("^(?s){}$", pattern)).unwrap())
  }

  pub(crate) fn assert_match(&self, output: &str) {
    match self {
      Self::String(string) => pretty_assertions::assert_eq!(output, string),
      Self::Regex(regex) => assert!(
        regex.is_match(output),
        "regex:\n{}\ndid not match output:\n{}",
        regex,
        output
      ),
      Self::Ignore => {}
    }
  }
}
