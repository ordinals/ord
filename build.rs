use {
  pulldown_cmark::{html, Parser},
  std::{env, fs, path::Path},
};

const MARKDOWN: &str = include_str!("faq.md");

fn main() {
  let parser = Parser::new(MARKDOWN);

  let mut output = String::new();
  html::push_html(&mut output, parser);

  fs::write(
    Path::new(&env::var("OUT_DIR").unwrap()).join("faq.html"),
    output,
  )
  .unwrap();
}
