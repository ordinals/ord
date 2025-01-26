use super::*;

#[derive(Boilerplate)]
pub(crate) struct SatscardHtml {
  pub(crate) satscard: Option<(Satscard, Option<AddressHtml>)>,
}

impl SatscardHtml {
  fn form_value(&self) -> Option<String> {
    self.satscard.as_ref().map(|(satscard, _address_info)| {
      format!("https://satscard.com/start#{}", satscard.query_parameters)
    })
  }
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
  fn form_value() {
    assert_eq!(
      SatscardHtml {
        satscard: Some((crate::satscard::tests::satscard(), None)),
      }
      .form_value(),
      Some(crate::satscard::tests::URL.into()),
    );

    assert_eq!(SatscardHtml { satscard: None }.form_value(), None);
  }

  #[test]
  fn title() {
    assert_eq!(
      SatscardHtml {
        satscard: Some((crate::satscard::tests::satscard(), None)),
      }
      .title(),
      format!("Satscard {}", crate::satscard::tests::address())
    );

    assert_eq!(SatscardHtml { satscard: None }.title(), "Satscard");
  }

  #[test]
  fn no_address_info() {
    pretty_assert_eq!(
      SatscardHtml {
        satscard: Some((crate::satscard::tests::satscard(), None)),
      }
      .to_string(),
      r#"<h1>Satscard bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf</h1>
<form>
  <label for=url>Satscard URL</label>
  <input
    type=text
    id=url
    name=url
    pattern='^https://(get)?satscard.com/start#.*$'
    required
    title='The URL should begin with "https://(get)satscard.com/start#".'
    value='https://satscard.com/start#u=S&amp;o=0&amp;r=a5x2tplf&amp;n=7664168a4ef7b8e8&amp;s=42b209c86ab90be6418d36b0accc3a53c11901861b55be95b763799842d403dc17cd1b74695a7ffe2d78965535d6fe7f6aafc77f6143912a163cb65862e8fb53'
  >
  <input type="submit" value="Submit">
</form>
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
          crate::satscard::tests::satscard(),
          Some(AddressHtml {
            address: crate::satscard::tests::address(),
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
    pattern='^https://(get)?satscard.com/start#.*$'
    required
    title='The URL should begin with "https://(get)satscard.com/start#".'
    value='https://satscard.com/start#u=S&amp;o=0&amp;r=a5x2tplf&amp;n=7664168a4ef7b8e8&amp;s=42b209c86ab90be6418d36b0accc3a53c11901861b55be95b763799842d403dc17cd1b74695a7ffe2d78965535d6fe7f6aafc77f6143912a163cb65862e8fb53'
  >
  <input type="submit" value="Submit">
</form>
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
            ..crate::satscard::tests::satscard()
          },
          Some(AddressHtml {
            address: crate::satscard::tests::address(),
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
            ..crate::satscard::tests::satscard()
          },
          Some(AddressHtml {
            address: crate::satscard::tests::address(),
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
