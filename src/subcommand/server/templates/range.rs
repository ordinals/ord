use super::*;

#[derive(Display)]
pub(crate) struct RangeHtml {
  pub(crate) start: Ordinal,
  pub(crate) end: Ordinal,
}

impl Content for RangeHtml {
  fn title(&self) -> String {
    format!("Ordinal range [{},{})", self.start, self.end)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, unindent::Unindent};

  #[test]
  fn range_html() {
    assert_eq!(
      RangeHtml {
        start: Ordinal(0),
        end: Ordinal(1),
      }
      .to_string(),
      "
        <h1>Ordinal range [0,1)</h1>
        <dl>
          <dt>size</dt><dd>1</dd>
          <dt>first</dt><dd><a href=/ordinal/0 class=mythic>0</a></dd>
        </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn bugfix_broken_link() {
    assert_eq!(
      RangeHtml {
        start: Ordinal(1),
        end: Ordinal(10),
      }
      .to_string(),
      "
        <h1>Ordinal range [1,10)</h1>
        <dl>
          <dt>size</dt><dd>9</dd>
          <dt>first</dt><dd><a href=/ordinal/1 class=common>1</a></dd>
        </dl>
      "
      .unindent()
    );
  }
}
