use super::*;
use crate::content_hash::ContentHash;

#[derive(Boilerplate)]
pub(crate) struct ContentHashHtml {
  hash: ContentHash,
  inscriptions: Vec<InscriptionId>,
}

impl ContentHashHtml {
  pub(crate) fn new(hash: ContentHash, inscriptions: Vec<InscriptionId>) -> Self {
    Self { hash, inscriptions }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ContentHashJson {
  hash: ContentHash,
  inscriptions: Vec<InscriptionId>,
}

impl ContentHashJson {
  pub fn new(hash: ContentHash, inscriptions: Vec<InscriptionId>) -> Self {
    Self { hash, inscriptions }
  }
}

impl PageContent for ContentHashHtml {
  fn title(&self) -> String {
    format!("Content Hash {}", self.hash)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn html() {
    assert_regex_match!(
      ContentHashHtml::new(
        ContentHash { hash: [0u8; 32] },
        vec![inscription_id(1), inscription_id(2)]
      ),
      "
        <h1>Content hash 0000000000000000000000000000000000000000000000000000000000000000</h1>
        <div class=inscription>
          <div>.*</div>
          <iframe sandbox=allow-scripts scrolling=no loading=lazy src=/preview/1111111111111111111111111111111111111111111111111111111111111111i1></iframe>
          <div>.*</div>
        </div>
        <dl>
          <dt>occurences</dt><dd>2</dd>
        </dl>
        <h2>2 Inscriptions</h2>
        <ul class=monospace>
          <li><a href=/inscription/1111111111111111111111111111111111111111111111111111111111111111i1>1111111111111111111111111111111111111111111111111111111111111111i1</a></li>
          <li><a href=/inscription/2222222222222222222222222222222222222222222222222222222222222222i2>2222222222222222222222222222222222222222222222222222222222222222i2</a></li>
        </ul>
      "
      .unindent()
    );
  }
}
