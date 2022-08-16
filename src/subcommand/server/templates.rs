use {super::*, boilerplate::Display};

#[derive(Display)]
pub(crate) struct OrdinalHtml {
  pub(crate) ordinal: Ordinal,
  pub(crate) blocktime: Blocktime,
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, unindent::Unindent};

  #[test]
  fn ordinal_html() {
    assert_eq!(
      OrdinalHtml {
        ordinal: Ordinal(0),
        blocktime: Blocktime::Confirmed(0),
      }
      .to_string(),
      "
        <!doctype html>
        <html>
          <head>
            <meta charset=utf-8>
            <title>0°0′0″0‴</title>
          </head>
          <body>
            <dl><dt>number</dt><dd>0</dd></dl>
            <dl><dt>decimal</dt><dd>0.0</dd></dl>
            <dl><dt>degree</dt><dd>0°0′0″0‴</dd></dl>
            <dl><dt>name</dt><dd>nvtdijuwxlp</dd></dl>
            <dl><dt>height</dt><dd>0</dd></dl>
            <dl><dt>cycle</dt><dd>0</dd></dl>
            <dl><dt>epoch</dt><dd>0</dd></dl>
            <dl><dt>period</dt><dd>0</dd></dl>
            <dl><dt>offset</dt><dd>0</dd></dl>
            <dl><dt>rarity</dt><dd>mythic</dd></dl>
            <dl><dt>block time</dt><dd>1970-01-01 00:00:00</dd></dl>
          </body>
        </html>
      "
      .unindent()
    );
  }
}
