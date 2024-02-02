use super::*;

#[derive(Boilerplate)]
pub(crate) struct RangeHtml {
  pub(crate) start: Sat,
  pub(crate) end: Sat,
}

impl PageContent for RangeHtml {
  fn title(&self) -> String {
    format!("Sat Range {}–{}", self.start, self.end)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn range_html() {
    pretty_assert_eq!(
      RangeHtml {
        start: Sat(0),
        end: Sat(1),
      }
      .to_string(),
      "
        <h1>Sat Range 0–1</h1>
        <dl>
          <dt>value</dt><dd>1</dd>
          <dt>first</dt><dd><a href=/sat/0 class=mythic>0</a></dd>
        </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn bugfix_broken_link() {
    pretty_assert_eq!(
      RangeHtml {
        start: Sat(1),
        end: Sat(10),
      }
      .to_string(),
      "
        <h1>Sat Range 1–10</h1>
        <dl>
          <dt>value</dt><dd>9</dd>
          <dt>first</dt><dd><a href=/sat/1 class=common>1</a></dd>
        </dl>
      "
      .unindent()
    );
  }
}
