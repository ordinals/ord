use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneNotFoundHtml {
  pub etchable: bool,
  pub reserved: bool,
  pub rune: Rune,
  pub unlock_height: Option<u32>,
}

impl PageContent for RuneNotFoundHtml {
  fn title(&self) -> String {
    format!("Rune {}", self.rune)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_regex_match!(
      RuneNotFoundHtml {
        etchable: true,
        reserved: false,
        rune: Rune(u128::MAX),
        unlock_height: Some(111),
      },
      "<h1>BCGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
<dl>
  <dt>unlock height</dt>
  <dd>111</dd>
  <dt>etchable</dt>
  <dd>true</dd>
</dl>
"
    );
  }
}
