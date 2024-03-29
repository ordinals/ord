use super::*;

#[derive(Debug)]
pub(crate) enum Expected {
  String(String),
  Regex(Regex),
}

impl Expected {
  pub(crate) fn regex(pattern: &str) -> Self {
    Self::Regex(Regex::new(&format!("^(?s){pattern}$")).unwrap())
  }

  #[track_caller]
  pub(crate) fn assert_match(&self, output: &str) {
    match self {
      Self::String(string) => pretty_assert_eq!(output, string),
      Self::Regex(regex) => {
        if !regex.is_match(output) {
          eprintln!("Regex did not match:");
          pretty_assert_eq!(regex.as_str(), output);
        }
      }
    }
  }
}
