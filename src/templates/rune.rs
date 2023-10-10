use super::*;

#[derive(Boilerplate)]
pub(crate) struct RuneHtml {
  pub(crate) entry: RuneEntry,
  pub(crate) id: RuneId,
  pub(crate) inscription: Option<InscriptionId>,
}

impl PageContent for RuneHtml {
  fn title(&self) -> String {
    format!("Rune {}", self.entry.rune)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, crate::runes::Rune};

  #[test]
  fn display() {
    assert_eq!(
      RuneHtml {
        entry: RuneEntry {
          divisibility: 9,
          rarity: Rarity::Uncommon,
          rune: Rune(u128::max_value()),
          supply: 123456789123456789,
          etching: Txid::all_zeros(),
        },
        id: RuneId {
          height: 10,
          index: 9,
        },
        inscription: Some(InscriptionId {
          txid: Txid::all_zeros(),
          index: 0,
        }),
      }
      .to_string(),
      "<h1>Rune BCGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
<dl>
  <dt>id</dt>
  <dd>10/9</dd>
  <dt>supply</dt>
  <dd>123456789.123456789</dd>
  <dt>divisibility</dt>
  <dd>9</dd>
  <dt>rarity</dt>
  <dd><span class=uncommon>uncommon</span></dd>
</dl>
"
    );
  }
}
