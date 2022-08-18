use {super::*, boilerplate::Display};

pub(crate) mod block;
pub(crate) mod home;
pub(crate) mod ordinal;
pub(crate) mod output;
pub(crate) mod range;
pub(crate) mod transaction;

#[derive(Display)]
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

    assert_eq!(
      Foo.page().to_string(),
      "<!doctype html>
<html lang=en>
  <head>
    <meta charset=utf-8>
    <meta name=format-detection content='telephone=no'>
    <meta name=viewport content='width=device-width,initial-scale=1.0'>
    <title>Foo</title>
    <link href=/static/index.css rel=stylesheet>
  </head>
  <body>
<h1>Foo</h1>
  </body>
</html>
"
    );
  }
}
