use {super::*, boilerplate::Boilerplate};

pub(crate) use {
  block::BlockHtml, clock::ClockSvg, content::ContentHtml, home::HomeHtml, input::InputHtml,
  inscription::InscriptionHtml, output::OutputHtml, range::RangeHtml, rare::RareTxt, sat::SatHtml,
  transaction::TransactionHtml,
};

mod block;
mod clock;
mod content;
mod home;
mod input;
mod inscription;
mod output;
mod range;
mod rare;
mod sat;
mod transaction;

#[derive(Boilerplate)]
pub(crate) struct PageHtml {
  chain: Chain,
  content: Box<dyn PageContent>,
  has_satoshi_index: bool,
}

impl PageHtml {
  pub(crate) fn new<T: PageContent + 'static>(
    content: T,
    chain: Chain,
    has_satoshi_index: bool,
  ) -> Self {
    Self {
      content: Box::new(content),
      has_satoshi_index,
      chain,
    }
  }
}

pub(crate) trait PageContent: Display + 'static {
  fn title(&self) -> String;

  fn page(self, chain: Chain, has_satoshi_index: bool) -> PageHtml
  where
    Self: Sized,
  {
    PageHtml::new(self, chain, has_satoshi_index)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn page_mainnet() {
    struct Foo;

    impl Display for Foo {
      fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<h1>Foo</h1>")
      }
    }

    impl PageContent for Foo {
      fn title(&self) -> String {
        "Foo".to_string()
      }
    }

    assert_regex_match!(
      Foo.page(Chain::Mainnet, true).to_string(),
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
      <a href=/clock>Clock</a>
      <a href=/rare.txt>rare.txt</a>
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

  #[test]
  fn page_no_satoshi_index() {
    struct Foo;

    impl Display for Foo {
      fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<h1>Foo</h1>")
      }
    }

    impl PageContent for Foo {
      fn title(&self) -> String {
        "Foo".to_string()
      }
    }

    assert_regex_match!(
      Foo.page(Chain::Mainnet, false).to_string(),
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
      <a href=/clock>Clock</a>
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

  #[test]
  fn page_signet() {
    struct Foo;

    impl Display for Foo {
      fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "<h1>Foo</h1>")
      }
    }

    impl PageContent for Foo {
      fn title(&self) -> String {
        "Foo".to_string()
      }
    }

    assert_regex_match!(
      Foo.page(Chain::Signet, true).to_string(),
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
      <a href=/>Ordinals<sup>signet</sup></a>
      .*
      <a href=/clock>Clock</a>
      <a href=/rare.txt>rare.txt</a>
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
