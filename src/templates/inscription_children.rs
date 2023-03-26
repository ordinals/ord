use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionChildrenHtml {
  pub(crate) parent_id: InscriptionId,
  pub(crate) parent_number: u64,
  pub(crate) children: Vec<InscriptionId>,
}

impl PageContent for InscriptionChildrenHtml {
  fn title(&self) -> String {
    format!("Children")
  }

  fn preview_image_url(&self) -> Option<Trusted<String>> {
    Some(Trusted(format!("/content/{}", self.parent_id)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_sat_or_nav_links() {
    assert_regex_match!(
      InscriptionChildrenHtml {
        parent_id: inscription_id(0),
        parent_number: 0,
        children: vec![inscription_id(1), inscription_id(2), inscription_id(3)],
      },
      "
        <h1>Parent inscription 0</h1>
        <a href=/inscription/0{64}i1><iframe .* src=/preview/0{64}i1></iframe></a>
        <h2>Children</h2>
        <div class=thumbnails>
          <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
          <a href=/inscription/3{64}i3><iframe .* src=/preview/3{64}i3></iframe></a>
        </div>
      "
      .unindent()
    );
  }
}
