use super::*;

#[derive(Display)]
pub(crate) struct HomeHtml {
  pub(crate) blocks: Vec<(u64, BlockHash)>,
}

impl Content for HomeHtml {
  fn title(&self) -> String {
    "Ordinals".to_string()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, regex::Regex};

  macro_rules! assert_regex_match {
    ($pattern:expr, $string:expr $(,)?) => {
      let regex = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
      let string = $string;

      if !regex.is_match(string) {
        panic!(
          "Regex:\n\n{}\n\n…did not match string:\n\n{}",
          regex, string
        );
      }
    };
  }

  #[test]
  fn home_html() {
    assert_regex_match!(
"<h1>Ordinals</h1>
<nav>.*</nav>
<h2>Recent Blocks</h2>
<ul>
  <li>1 - <a href=/block/1111111111111111111111111111111111111111111111111111111111111111>1111111111111111111111111111111111111111111111111111111111111111</a></li>
  <li>0 - <a href=/block/0000000000000000000000000000000000000000000000000000000000000000>0000000000000000000000000000000000000000000000000000000000000000</a></li>
</ul>
",
      &HomeHtml {
        blocks: vec![
          (
            1,
            "1111111111111111111111111111111111111111111111111111111111111111".parse().unwrap()
          ),
          (
            0,
            "0000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()
          )
        ],
      }.to_string());
  }
}
