use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionChildrenHtml {
  pub(crate) parent_inscription_id: InscriptionId,
  pub(crate) children: Vec<InscriptionId>,
  pub(crate) prev_page: Option<usize>,
  pub(crate) next_page: Option<usize>,
}

impl InscriptionChildrenHtml {
  pub(crate) fn new(
    parent_inscription_id: InscriptionId,
    children: Vec<InscriptionId>,
    page_index: usize,
  ) -> Result<Self> {
    let num_children = children.len();

    let start = page_index * 20;
    let end = usize::min(start + 20, num_children);

    if start > num_children || start > end {
      return Err(anyhow!("page index {page_index} exceeds inscription count"));
    }
    let children = children[start..end].to_vec();

    Ok(Self {
      parent_inscription_id,
      children,
      prev_page: if page_index > 0 {
        Some(page_index - 1)
      } else {
        None
      },
      next_page: if (page_index + 1) * 20 <= num_children {
        Some(page_index + 1)
      } else {
        None
      },
    })
  }
}

impl PageContent for InscriptionChildrenHtml {
  fn title(&self) -> String {
    format!("Inscription {} Children", self.parent_inscription_id)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_prev_and_next() {
    assert_regex_match!(
      InscriptionChildrenHtml {
        parent_inscription_id: inscription_id(1),
        children: vec![inscription_id(2), inscription_id(3)],
        prev_page: None,
        next_page: None,
      },
      "
        <h1 class=light-fg>Children of Inscription <a href=/inscription/1{64}i1>1{64}i1</a></h1>
        <div class=thumbnails>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
          <a href=/inscription/3{64}i3><iframe .* src=/preview/3{64}i3></iframe></a>
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
      InscriptionChildrenHtml {
        parent_inscription_id: inscription_id(1),
        children: vec![inscription_id(2), inscription_id(3)],
        next_page: Some(3),
        prev_page: Some(1),
      },
      "
        <h1 class=light-fg>Children of Inscription <a href=/inscription/1{64}i1>1{64}i1</a></h1>
        <div class=thumbnails>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
          <a href=/inscription/3{64}i3><iframe .* src=/preview/3{64}i3></iframe></a>
        </div>
        .*
          <a class=prev href=/inscription/1{64}i1/children/1>prev</a>
          <a class=next href=/inscription/1{64}i1/children/3>next</a>
        .*
      "
      .unindent()
    );
  }
}
