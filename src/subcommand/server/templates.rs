use {
  super::*,
  boilerplate::Boilerplate,
  html_escaper::{Escape, Trusted},
};

pub(crate) use {
  block::BlockHtml, clock::ClockSvg, home::HomeHtml, input::InputHtml, ordinal::OrdinalHtml,
  output::OutputHtml, range::RangeHtml, rare::RareTxt, rune::RuneHtml,
  transaction::TransactionHtml,
};

mod block;
mod clock;
mod home;
mod input;
mod ordinal;
mod output;
mod range;
mod rare;
mod rune;
mod transaction;

#[derive(Boilerplate)]
pub(crate) struct PageHtml {
  content: Box<dyn Content>,
}

impl PageHtml {
  pub(crate) fn new<T: Content + 'static>(content: T) -> Self {
    Self {
      content: Box::new(content),
    }
  }
}

pub(crate) trait Content: Display + 'static {
  fn title(&self) -> String;

  fn page(self) -> PageHtml
  where
    Self: Sized,
  {
    PageHtml::new(self)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn page() {
    struct Foo;

    impl Display for Foo {
      fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<h1>Foo</h1>")
      }
    }

    impl Content for Foo {
      fn title(&self) -> String {
        "Foo".to_string()
      }
    }

    assert_regex_match!(
      Foo.page().to_string(),
      "<!doctype html>
<html lang=en>
  <head>
    <meta charset=utf-8>
    <meta name=format-detection content='telephone=no'>
    <meta name=viewport content='width=device-width,initial-scale=1.0'>
    <title>Foo</title>
    <link href=/static/index.css rel=stylesheet>
    <link href=/static/modern-normalize.css rel=stylesheet>
  </head>
  <body>
  <header>
    <nav>
      <a href=/>Ordinals</a>
      .*
      <form action=/search method=get>
        <input type=text .*>
        <input type=submit value=Search>
      </form>
    </nav>
  </header>
  <main>
<h1>Foo</h1>
  </main>
  </body>
</html>
"
    );
  }
}
