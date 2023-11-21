use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionsHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) prev: Option<u32>,
  pub(crate) next: Option<u32>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InscriptionsJson {
  pub inscriptions: Vec<InscriptionId>,
  pub prev: Option<u32>,
  pub next: Option<u32>,
  pub lowest: Option<u32>,
  pub highest: Option<u32>,
}

impl InscriptionsJson {
  pub fn new(
    inscriptions: Vec<InscriptionId>,
    prev: Option<u32>,
    next: Option<u32>,
    lowest: Option<u32>,
    highest: Option<u32>,
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
