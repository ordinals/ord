pub(crate) use regex::Regex;

macro_rules! assert_regex_match {
  ($string:expr, $pattern:expr $(,)?) => {
    let pattern: &'static str = $pattern;
    let regex = Regex::new(&format!("^(?s){}$", pattern)).unwrap();
    let string = $string;

    if !regex.is_match(string.as_ref()) {
      panic!(
        "Regex:\n\n{}\n\n…did not match string:\n\n{}",
        regex, string
      );
    }
  };
}
