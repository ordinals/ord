use super::*;

#[derive(Boilerplate)]
pub(crate) struct HomeHtml {
  pub(crate) all: Vec<InscriptionId>,
  pub(crate) collections: Vec<InscriptionId>,
  pub(crate) galleries: Vec<InscriptionId>,
  pub(crate) latest: Vec<InscriptionId>,
  pub(crate) runes: Vec<InscriptionId>,
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
        all: vec![inscription_id(5)],
        collections: vec![inscription_id(2)],
        galleries: vec![inscription_id(4)],
        latest: vec![inscription_id(1)],
        runes: vec![inscription_id(3)],
      }
      .to_string()
      .unindent(),
      "<h2><a href=/latest>Latest Inscriptions</a></h2>
      <div class=thumbnails>
        <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
      </div>
      <h2><a href=/collections>Collections</a></h2>
      <div class=thumbnails>
        <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
      </div>
      <h2><a href=/runes>Runes</a></h2>
      <div class=thumbnails>
        <a href=/inscription/3{64}i3><iframe .* src=/preview/3{64}i3></iframe></a>
      </div>
      <h2><a href=/galleries>Galleries</a></h2>
      <div class=thumbnails>
        <a href=/inscription/4{64}i4><iframe .* src=/preview/4{64}i4></iframe></a>
      </div>
      <h2><a href=/inscriptions>All Inscriptions</a></h2>
      <div class=thumbnails>
        <a href=/inscription/5{64}i5><iframe .* src=/preview/5{64}i5></iframe></a>
      </div>
      "
      .unindent(),
    );
  }
}
