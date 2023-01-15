use {super::*, boilerplate::Boilerplate};

pub(crate) use {
  block::BlockHtml,
  clock::ClockSvg,
  home::HomeHtml,
  iframe::Iframe,
  input::InputHtml,
  inscription::InscriptionHtml,
  inscriptions::InscriptionsHtml,
  output::OutputHtml,
  preview::{PreviewAudioHtml, PreviewImageHtml, PreviewTextHtml, PreviewUnknownHtml},
  range::RangeHtml,
  rare::RareTxt,
  sat::SatHtml,
  transaction::TransactionHtml,
};

mod block;
mod clock;
mod home;
mod iframe;
mod input;
mod inscription;
mod inscriptions;
mod output;
mod preview;
mod range;
mod rare;
mod sat;
mod transaction;

#[derive(Boilerplate)]
pub(crate) struct PageHtml<T: PageContent> {
  chain: Chain,
  content: T,
  has_sat_index: bool,
}

impl<T> PageHtml<T>
where
  T: PageContent,
{
  pub(crate) fn new(content: T, chain: Chain, has_sat_index: bool) -> Self {
    Self {
      content,
      has_sat_index,
      chain,
    }
  }
}

pub(crate) trait PageContent: Display + 'static {
  fn title(&self) -> String;

  fn page(self, chain: Chain, has_sat_index: bool) -> PageHtml<Self>
  where
    Self: Sized,
  {
    PageHtml::new(self, chain, has_sat_index)
  }

  fn preview_image_url(&self) -> Option<Trusted<String>> {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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

  #[test]
  fn page() {
    assert_regex_match!(
      Foo.page(Chain::Mainnet, true),
      "<!doctype html>
<html lang=en>
  <head>
    <meta charset=utf-8>
    <meta name=format-detection content='telephone=no'>
    <meta name=viewport content='width=device-width,initial-scale=1.0'>
    <title>Foo</title>
    <link href=/static/index.css rel=stylesheet>
    <link href=/static/modern-normalize.css rel=stylesheet>
    <script src=/static/index.js defer></script>
  </head>
  <body>
  <header>
    <nav>
      <a href=/>Ordinals<sup>alpha</sup></a>
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
  fn page_mainnet() {
    assert_regex_match!(
      Foo.page(Chain::Mainnet, true),
      r".*<nav>\s*<a href=/>Ordinals<sup>alpha</sup></a>.*"
    );
  }

  #[test]
  fn page_no_sat_index() {
    assert_regex_match!(
      Foo.page(Chain::Mainnet, false),
      r".*<nav>\s*<a href=/>Ordinals<sup>alpha</sup></a>.*<a href=/clock>Clock</a>\s*<form action=/search.*",
    );
  }

  #[test]
  fn page_signet() {
    assert_regex_match!(
      Foo.page(Chain::Signet, true),
      r".*<nav>\s*<a href=/>Ordinals<sup>signet</sup></a>.*"
    );
  }
}
