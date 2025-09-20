use super::*;

#[derive(Boilerplate)]
pub(crate) struct SatscardHtml {
  pub(crate) satscard: Option<(Satscard, Option<AddressHtml>)>,
}

impl PageContent for SatscardHtml {
  fn title(&self) -> String {
    if let Some((satscard, _address_info)) = &self.satscard {
      format!("Satscard {}", satscard.address)
    } else {
      "Satscard".into()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn title() {
    assert_eq!(
      SatscardHtml {
        satscard: Some((crate::satscard::tests::coinkite_satscard(), None)),
      }
      .title(),
      format!("Satscard {}", crate::satscard::tests::coinkite_address())
    );

    assert_eq!(SatscardHtml { satscard: None }.title(), "Satscard");
  }

  #[test]
  fn no_address_info() {
    pretty_assert_eq!(
      SatscardHtml {
        satscard: Some((crate::satscard::tests::coinkite_satscard(), None)),
      }
      .to_string(),
      r#"<h1>Satscard bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf</h1>
<form>
  <label for=url>Satscard URL</label>
  <input
    type=text
    id=url
    name=url
    required
  >
  <input type="submit" value="Submit">
</form>
<p>
  <a href=https://docs.ordinals.com/guides/satscards.html>Guide</a>
</p>
<dl>
  <dt>slot</dt>
  <dd>1</dd>
  <dt>state</dt>
  <dd class=satscard-sealed>sealed</dd>
  <dt>address</dt>
  <dd><a class=collapse href=/address/bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf>bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf</a></dd>
  <dt>nonce</dt>
  <dd>7664168a4ef7b8e8</dd>
</dl>
"#,
    );
  }

  #[test]
  fn with_address_info() {
    pretty_assert_eq!(
      SatscardHtml {
        satscard: Some((
          crate::satscard::tests::coinkite_satscard(),
          Some(AddressHtml {
            address: crate::satscard::tests::coinkite_address(),
            header: false,
            inscriptions: Some(Vec::new()),
            outputs: Vec::new(),
            runes_balances: None,
            sat_balance: 0,
          })
        )),
      }
      .to_string(),
      r#"<h1>Satscard bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf</h1>
<form>
  <label for=url>Satscard URL</label>
  <input
    type=text
    id=url
    name=url
    required
  >
  <input type="submit" value="Submit">
</form>
<p>
  <a href=https://docs.ordinals.com/guides/satscards.html>Guide</a>
</p>
<dl>
  <dt>slot</dt>
  <dd>1</dd>
  <dt>state</dt>
  <dd class=satscard-sealed>sealed</dd>
  <dt>address</dt>
  <dd><a class=collapse href=/address/bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf>bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf</a></dd>
  <dt>nonce</dt>
  <dd>7664168a4ef7b8e8</dd>
</dl>
<dl>
  <dt>sat balance</dt>
  <dd>0</dd>
  <dt>outputs</dt>
  <dd>
    <ul>
    </ul>
  </dd>
</dl>

"#,
    );
  }

  #[test]
  fn state_error() {
    assert_regex_match! {
      SatscardHtml {
        satscard: Some((
          Satscard {
            state: crate::satscard::State::Error,
            ..crate::satscard::tests::coinkite_satscard()
          },
          Some(AddressHtml {
            address: crate::satscard::tests::coinkite_address(),
            header: false,
            inscriptions: Some(Vec::new()),
            outputs: Vec::new(),
            runes_balances: None,
            sat_balance: 0,
          })
        )),
      }
      .to_string(),
      r#".*
  <dt>state</dt>
  <dd class=satscard-error>error</dd>
.*
"#,
    }
  }

  #[test]
  fn state_unsealed() {
    assert_regex_match! {
      SatscardHtml {
        satscard: Some((
          Satscard {
            state: crate::satscard::State::Unsealed,
            ..crate::satscard::tests::coinkite_satscard()
          },
          Some(AddressHtml {
            address: crate::satscard::tests::coinkite_address(),
            header: false,
            inscriptions: Some(Vec::new()),
            outputs: Vec::new(),
            runes_balances: None,
            sat_balance: 0,
          })
        )),
      }
      .to_string(),
      r#".*
  <dt>state</dt>
  <dd class=satscard-unsealed>unsealed</dd>
.*
"#,
    }
  }
}
