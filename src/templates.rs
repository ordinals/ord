use {super::*, boilerplate::Boilerplate};

pub(crate) use {
  block::BlockHtml,
  clock::ClockSvg,
  home::HomeHtml,
  iframe::Iframe,
  input::InputHtml,
  inscription::{InscriptionHtml, InscriptionJson},
  inscriptions::{InscriptionsHtml, InscriptionsJson},
  output::{OutputHtml, OutputJson},
  page_config::PageConfig,
  preview::{
    PreviewAudioHtml, PreviewImageHtml, PreviewPdfHtml, PreviewTextHtml, PreviewUnknownHtml,
    PreviewVideoHtml,
  },
  range::RangeHtml,
  rare::RareTxt,
  sat::{SatHtml, SatJson},
  transaction::TransactionHtml,
};

mod block;
mod clock;
mod home;
mod iframe;
mod input;
pub mod inscription;
pub mod inscriptions;
pub mod output;
mod preview;
mod range;
mod rare;
pub mod sat;
mod transaction;

#[derive(Boilerplate)]
pub(crate) struct PageHtml<T: PageContent> {
  content: T,
  has_sat_index: bool,
  page_config: Arc<PageConfig>,
}

impl<T> PageHtml<T>
where
  T: PageContent,
{
  pub(crate) fn new(content: T, page_config: Arc<PageConfig>, has_sat_index: bool) -> Self {
    Self {
      content,
      has_sat_index,
      page_config,
    }
  }

  fn og_image(&self) -> String {
    if let Some(domain) = &self.page_config.domain {
      format!("https://{domain}/static/favicon.png")
    } else {
      "https://ordinals.com/static/favicon.png".into()
    }
  }

  fn superscript(&self) -> String {
    if self.page_config.chain == Chain::Mainnet {
      "alpha".into()
    } else {
      self.page_config.chain.to_string()
    }
  }
}

pub(crate) trait PageContent: Display + 'static {
  fn title(&self) -> String;

  fn page(self, page_config: Arc<PageConfig>, has_sat_index: bool) -> PageHtml<Self>
  where
    Self: Sized,
  {
    PageHtml::new(self, page_config, has_sat_index)
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
      Foo.page(
        Arc::new(PageConfig {
          chain: Chain::Mainnet,
          domain: Some("signet.ordinals.com".into())
        }),
        true
      ),
      r"<!doctype html>
<html lang=en>
  <head>
    <meta charset=utf-8>
    <meta name=format-detection content='telephone=no'>
    <meta name=viewport content='width=device-width,initial-scale=1.0'>
    <meta property=og:title content='Foo'>
    <meta property=og:image content='https://signet.ordinals.com/static/favicon.png'>
    <meta property=twitter:card content=summary>
    <title>Foo</title>
    <link rel=alternate href=/feed.xml type=application/rss\+xml title='Inscription RSS Feed'>
    <link rel=stylesheet href=/static/index.css>
    <link rel=stylesheet href=/static/modern-normalize.css>
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
        <input type=submit value='&#9906'>
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
      Foo.page(
        Arc::new(PageConfig {
          chain: Chain::Mainnet,
          domain: None
        }),
        true
      ),
      r".*<nav>\s*<a href=/>Ordinals<sup>alpha</sup></a>.*"
    );
  }

  #[test]
  fn page_no_sat_index() {
    assert_regex_match!(
      Foo.page(
        Arc::new(PageConfig {
          chain: Chain::Mainnet,
          domain: None
        }),
        false
      ),
      r".*<nav>\s*<a href=/>Ordinals<sup>alpha</sup></a>.*<a href=/clock>Clock</a>\s*<form action=/search.*",
    );
  }

  #[test]
  fn page_signet() {
    assert_regex_match!(
      Foo.page(
        Arc::new(PageConfig {
          chain: Chain::Signet,
          domain: None
        }),
        true
      ),
      r".*<nav>\s*<a href=/>Ordinals<sup>signet</sup></a>.*"
    );
  }
}
