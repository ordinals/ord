use super::*;

#[derive(Boilerplate)]
pub(crate) struct HomeHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
}

impl PageContent for HomeHtml {
  fn title(&self) -> String {
    "Ordinals".to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn html() {
    assert_regex_match!(
      HomeHtml {
        inscriptions: vec![inscription_id(1), inscription_id(2)],
      }
      .to_string()
      .unindent(),
      "<h1>Latest Inscriptions</h1>
      <div class=thumbnails>
        <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
        <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
      </div>
      "
      .unindent(),
    );
  }
}
