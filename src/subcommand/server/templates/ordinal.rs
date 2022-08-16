use super::*;

#[derive(Display)]
pub(crate) struct OrdinalHtml {
  pub(crate) ordinal: Ordinal,
  pub(crate) blocktime: Blocktime,
}

impl Page for OrdinalHtml {
  fn title(&self) -> String {
    self.ordinal.degree().to_string()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, unindent::Unindent};

  #[test]
  fn ordinal_html() {
    assert_eq!(
      BaseHtml::new(OrdinalHtml {
        ordinal: Ordinal(0),
        blocktime: Blocktime::Confirmed(0),
      })
      .to_string(),
      "
        <!doctype html>
        <html lang=en>
          <head>
            <meta charset=utf-8>
            <title>0°0′0″0‴</title>
          </head>
          <body>
            <dl>
              <dt>number</dt><dd>0</dd>
              <dt>decimal</dt><dd>0.0</dd>
              <dt>degree</dt><dd>0°0′0″0‴</dd>
              <dt>name</dt><dd>nvtdijuwxlp</dd>
              <dt>height</dt><dd>0</dd>
              <dt>cycle</dt><dd>0</dd>
              <dt>epoch</dt><dd>0</dd>
              <dt>period</dt><dd>0</dd>
              <dt>offset</dt><dd>0</dd>
              <dt>rarity</dt><dd>mythic</dd>
              <dt>block time</dt><dd>1970-01-01 00:00:00</dd>
            </dl>

          </body>
        </html>
      "
      .unindent()
    );
  }
}
