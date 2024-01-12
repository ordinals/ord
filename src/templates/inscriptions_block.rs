use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionsBlockHtml {
  pub(crate) block: u32,
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) prev_block: Option<u32>,
  pub(crate) next_block: Option<u32>,
  pub(crate) prev_page: Option<u32>,
  pub(crate) next_page: Option<u32>,
}

impl InscriptionsBlockHtml {
  pub(crate) fn new(
    block: u32,
    current_blockheight: u32,
    inscriptions: Vec<InscriptionId>,
    more_inscriptions: bool,
    page_index: u32,
  ) -> Result<Self> {
    if inscriptions.is_empty() {
      return Err(anyhow!("page index {page_index} exceeds inscription count"));
    }

    Ok(Self {
      block,
      inscriptions,
      prev_block: block.checked_sub(1),
      next_block: if current_blockheight > block {
        Some(block + 1)
      } else {
        None
      },
      prev_page: page_index.checked_sub(1),
      next_page: if more_inscriptions {
        Some(page_index + 1)
      } else {
        None
      },
    })
  }
}

impl PageContent for InscriptionsBlockHtml {
  fn title(&self) -> String {
    format!("Inscriptions in Block {0}", self.block)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_prev_and_next() {
    assert_regex_match!(
      InscriptionsBlockHtml {
        block: 21,
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev_block: None,
        next_block: None,
        prev_page: None,
        next_page: None,
      },
      "
        <h1>Inscriptions in <a href=/block/21>Block 21</a></h1>
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
      InscriptionsBlockHtml {
        block: 21,
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev_block: Some(20),
        next_block: Some(22),
        next_page: Some(3),
        prev_page: Some(1),
      },
      "
        <h1>Inscriptions in <a href=/block/21>Block 21</a></h1>
        <div class=thumbnails>
          <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
        </div>
        .*
          <a class=prev href=/inscriptions/block/20>20</a>
        &bull;
          <a class=prev href=/inscriptions/block/21/1>prev</a>
          <a class=next href=/inscriptions/block/21/3>next</a>
        &bull;
          <a class=next href=/inscriptions/block/22>22</a>
        .*
      "
      .unindent()
    );
  }
}
