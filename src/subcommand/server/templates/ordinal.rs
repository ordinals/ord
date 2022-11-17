use super::*;

#[derive(Boilerplate)]
pub(crate) struct OrdinalHtml {
  pub(crate) ordinal: Ordinal,
  pub(crate) blocktime: Blocktime,
  pub(crate) inscription: Option<Inscription>,
}

impl Content for OrdinalHtml {
  fn title(&self) -> String {
    self.ordinal.degree().to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ordinal_html() {
    pretty_assert_eq!(
      OrdinalHtml {
        ordinal: Ordinal(0),
        blocktime: Blocktime::Confirmed(0),
        inscription: None,
      }
      .to_string(),
      "
        <h1>Ordinal 0</h1>
        <dl>
          <dt>decimal</dt><dd>0.0</dd>
          <dt>degree</dt><dd>0°0′0″0‴</dd>
          <dt>percentile</dt><dd>0%</dd>
          <dt>name</dt><dd>nvtdijuwxlp</dd>
          <dt>cycle</dt><dd>0</dd>
          <dt>epoch</dt><dd>0</dd>
          <dt>period</dt><dd>0</dd>
          <dt>block</dt><dd>0</dd>
          <dt>offset</dt><dd>0</dd>
          <dt>rarity</dt><dd><span class=mythic>mythic</span></dd>
          <dt>time</dt><dd>1970-01-01 00:00:00</dd>
        </dl>
        prev
        <a href=/ordinal/1>next</a>
      "
      .unindent()
    );
  }

  #[test]
  fn ordinal_next_and_previous() {
    pretty_assert_eq!(
      OrdinalHtml {
        ordinal: Ordinal(1),
        blocktime: Blocktime::Confirmed(0),
        inscription: None,
      }
      .to_string(),
      "
        <h1>Ordinal 1</h1>
        <dl>
          <dt>decimal</dt><dd>0.1</dd>
          <dt>degree</dt><dd>0°0′0″1‴</dd>
          <dt>percentile</dt><dd>0.000000000000047619047671428595%</dd>
          <dt>name</dt><dd>nvtdijuwxlo</dd>
          <dt>cycle</dt><dd>0</dd>
          <dt>epoch</dt><dd>0</dd>
          <dt>period</dt><dd>0</dd>
          <dt>block</dt><dd>0</dd>
          <dt>offset</dt><dd>1</dd>
          <dt>rarity</dt><dd><span class=common>common</span></dd>
          <dt>time</dt><dd>1970-01-01 00:00:00</dd>
        </dl>
        <a href=/ordinal/0>prev</a>
        <a href=/ordinal/2>next</a>
      "
      .unindent()
    );
  }

  #[test]
  fn ordinal_with_inscription() {
    pretty_assert_eq!(
      OrdinalHtml {
        ordinal: Ordinal(0),
        blocktime: Blocktime::Confirmed(0),
        inscription: Some(Inscription::Text("HELLOWORLD".into())),
      }
      .to_string(),
      "
        <h1>Ordinal 0</h1>
        <dl>
          <dt>decimal</dt><dd>0.0</dd>
          <dt>degree</dt><dd>0°0′0″0‴</dd>
          <dt>percentile</dt><dd>0%</dd>
          <dt>name</dt><dd>nvtdijuwxlp</dd>
          <dt>cycle</dt><dd>0</dd>
          <dt>epoch</dt><dd>0</dd>
          <dt>period</dt><dd>0</dd>
          <dt>block</dt><dd>0</dd>
          <dt>offset</dt><dd>0</dd>
          <dt>rarity</dt><dd><span class=mythic>mythic</span></dd>
          <dt>time</dt><dd>1970-01-01 00:00:00</dd>
          <dt>inscription</dt><dd>HELLOWORLD</dd>
        </dl>
        prev
        <a href=/ordinal/1>next</a>
      "
      .unindent()
    );
  }

  #[test]
  fn ordinal_inscriptions_are_escaped() {
    pretty_assert_eq!(
      OrdinalHtml {
        ordinal: Ordinal(0),
        blocktime: Blocktime::Confirmed(0),
        inscription: Some(Inscription::Text(
          "<script>alert('HELLOWORLD');</script>".into()
        )),
      }
      .to_string(),
      "
        <h1>Ordinal 0</h1>
        <dl>
          <dt>decimal</dt><dd>0.0</dd>
          <dt>degree</dt><dd>0°0′0″0‴</dd>
          <dt>percentile</dt><dd>0%</dd>
          <dt>name</dt><dd>nvtdijuwxlp</dd>
          <dt>cycle</dt><dd>0</dd>
          <dt>epoch</dt><dd>0</dd>
          <dt>period</dt><dd>0</dd>
          <dt>block</dt><dd>0</dd>
          <dt>offset</dt><dd>0</dd>
          <dt>rarity</dt><dd><span class=mythic>mythic</span></dd>
          <dt>time</dt><dd>1970-01-01 00:00:00</dd>
          <dt>inscription</dt><dd>&lt;script&gt;alert(&apos;HELLOWORLD&apos;);&lt;/script&gt;</dd>
        </dl>
        prev
        <a href=/ordinal/1>next</a>
      "
      .unindent()
    );
  }

  #[test]
  fn last_ordinal_next_link_is_disabled() {
    pretty_assert_eq!(
      OrdinalHtml {
        ordinal: Ordinal::LAST,
        blocktime: Blocktime::Confirmed(0),
        inscription: None,
      }
      .to_string(),
      "
        <h1>Ordinal 2099999997689999</h1>
        <dl>
          <dt>decimal</dt><dd>6929999.0</dd>
          <dt>degree</dt><dd>5°209999′1007″0‴</dd>
          <dt>percentile</dt><dd>100%</dd>
          <dt>name</dt><dd>a</dd>
          <dt>cycle</dt><dd>5</dd>
          <dt>epoch</dt><dd>32</dd>
          <dt>period</dt><dd>3437</dd>
          <dt>block</dt><dd>6929999</dd>
          <dt>offset</dt><dd>0</dd>
          <dt>rarity</dt><dd><span class=uncommon>uncommon</span></dd>
          <dt>time</dt><dd>1970-01-01 00:00:00</dd>
        </dl>
        <a href=/ordinal/2099999997689998>prev</a>
        next
      "
      .unindent()
    );
  }
}
