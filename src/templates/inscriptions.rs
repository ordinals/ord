use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionsHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) prev: Option<i64>,
  pub(crate) next: Option<i64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InscriptionsJson {
  pub inscriptions: Vec<InscriptionId>,
  pub prev: Option<i64>,
  pub next: Option<i64>,
  pub lowest: Option<i64>,
  pub highest: Option<i64>,
}

impl InscriptionsJson {
  pub fn new(
    inscriptions: Vec<InscriptionId>,
    prev: Option<i64>,
    next: Option<i64>,
    lowest: Option<i64>,
    highest: Option<i64>,
  ) -> Self {
    Self {
      inscriptions,
      prev,
      next,
      lowest,
      highest,
    }
  }
}

impl PageContent for InscriptionsHtml {
  fn title(&self) -> String {
    "Inscriptions".into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_prev_and_next() {
    assert_regex_match!(
      InscriptionsHtml {
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev: None,
        next: None,
      },
      "
        <h1>Inscriptions</h1>
        <div class=thumbnails>
          <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
        </div>
        .*
        prev
        next
        .*
      "
      .unindent()
    );
  }

  #[test]
  fn with_prev_and_next() {
    assert_regex_match!(
      InscriptionsHtml {
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev: Some(1),
        next: Some(2),
      },
      "
        <h1>Inscriptions</h1>
        <div class=thumbnails>
          <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
        </div>
        .*
        <a class=prev href=/inscriptions/1>prev</a>
        <a class=next href=/inscriptions/2>next</a>
        .*
      "
      .unindent()
    );
  }
}
