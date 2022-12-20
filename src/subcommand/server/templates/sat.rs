use super::*;

#[derive(Boilerplate)]
pub(crate) struct SatHtml {
  pub(crate) sat: Sat,
  pub(crate) blocktime: Blocktime,
  pub(crate) inscription: Option<(InscriptionId, Inscription)>,
}

impl PageContent for SatHtml {
  fn title(&self) -> String {
    self.sat.degree().to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sat_html() {
    pretty_assert_eq!(
      SatHtml {
        sat: Sat(0),
        blocktime: Blocktime::Confirmed(0),
        inscription: None,
      }
      .to_string(),
      "
        <h1>Sat 0</h1>
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
        <a href=/sat/1>next</a>
      "
      .unindent()
    );
  }

  #[test]
  fn sat_next_and_previous() {
    pretty_assert_eq!(
      SatHtml {
        sat: Sat(1),
        blocktime: Blocktime::Confirmed(0),
        inscription: None,
      }
      .to_string(),
      "
        <h1>Sat 1</h1>
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
        <a href=/sat/0>prev</a>
        <a href=/sat/2>next</a>
      "
      .unindent()
    );
  }

  #[test]
  fn sat_with_inscription() {
    pretty_assert_eq!(
      SatHtml {
        sat: Sat(0),
        blocktime: Blocktime::Confirmed(0),
        inscription: Some((
          InscriptionId::from_str(
            "ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc"
          )
          .unwrap(),
          inscription("text/plain;charset=utf-8", "HELLOWORLD")
        )),
      }
      .to_string(),
      "
        <h1>Sat 0</h1>
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
          <dt>inscription</dt>
          <dd><p>HELLOWORLD</p></dd>
        </dl>
        prev
        <a href=/sat/1>next</a>
      "
      .unindent()
    );
  }

  #[test]
  fn sat_inscriptions_are_escaped() {
    pretty_assert_eq!(
      SatHtml {
        sat: Sat(0),
        blocktime: Blocktime::Confirmed(0),
        inscription: Some((
          InscriptionId::from_str(
            "ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc"
          )
          .unwrap(),
          inscription(
            "text/plain;charset=utf-8",
            "<script>alert('HELLOWORLD');</script>",
          )
        )),
      }
      .to_string(),
      "
        <h1>Sat 0</h1>
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
          <dt>inscription</dt>
          <dd><p>&lt;script&gt;alert(&apos;HELLOWORLD&apos;);&lt;/script&gt;</p></dd>
        </dl>
        prev
        <a href=/sat/1>next</a>
      "
      .unindent()
    );
  }

  #[test]
  fn last_sat_next_link_is_disabled() {
    pretty_assert_eq!(
      SatHtml {
        sat: Sat::LAST,
        blocktime: Blocktime::Confirmed(0),
        inscription: None,
      }
      .to_string(),
      "
        <h1>Sat 2099999997689999</h1>
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
        <a href=/sat/2099999997689998>prev</a>
        next
      "
      .unindent()
    );
  }
}
