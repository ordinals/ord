use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize)]
pub struct RuneNotFoundHtml {
  pub rune: Rune,
  pub unlock_height: Option<Height>,
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
        rune: Rune(u128::MAX),
        unlock_height: Some(Height(111)),
      },
      "<h1>BCGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
<dl>
  <dt>unlock height</dt>
  <dd>111</dd>
  <dt>reserved</dt>
  <dd>false</dd>
</dl>
"
    );
  }

  #[test]
  fn display_reserved() {
    assert_regex_match!(
      RuneNotFoundHtml {
        rune: Rune(Rune::RESERVED),
        unlock_height: None,
      },
      "<h1>AAAAAAAAAAAAAAAAAAAAAAAAAAA</h1>
<dl>
  <dt>unlock height</dt>
  <dd>none</dd>
  <dt>reserved</dt>
  <dd>true</dd>
</dl>
"
    );
  }
}
