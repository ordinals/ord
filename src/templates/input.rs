use super::*;

#[derive(Boilerplate)]
pub(crate) struct InputHtml {
  pub(crate) path: (u32, usize, usize),
  pub(crate) input: TxIn,
}

impl PageContent for InputHtml {
  fn title(&self) -> String {
    format!("Input /{}/{}/{}", self.path.0, self.path.1, self.path.2)
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    bitcoin::{blockdata::script, Witness},
  };

  #[test]
  fn input_html() {
    let mut witness = Witness::new();
    witness.push([1]);
    pretty_assert_eq!(
      InputHtml {
        path: (1, 2, 3),
        input: TxIn {
          previous_output: "0000000000000000000000000000000000000000000000000000000000000000:0"
            .parse()
            .unwrap(),
          script_sig: ScriptBuf::builder().push_slice(b"foo").into_script(),
          sequence: Sequence::MAX,
          witness,
        }
      }
      .to_string(),
      "
      <h1>Input /1/2/3</h1>
      <dl>
        <dt>previous output</dt><dd class=collapse>0000000000000000000000000000000000000000000000000000000000000000:0</dd>
        <dt>witness</dt><dd class=monospace>010101</dd>
        <dt>script sig</dt><dd class=monospace>OP_PUSHBYTES_3 666f6f</dd>
        <dt>text</dt><dd>\x03foo</dd>
      </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn skip_empty_items() {
    pretty_assert_eq!(
      InputHtml {
        path: (1, 2, 3),
        input: TxIn {
          previous_output: OutPoint::null(),
          script_sig: script::Builder::new().into_script(),
          sequence: Sequence::MAX,
          witness: Witness::new(),
        }
      }
      .to_string(),
      "
      <h1>Input /1/2/3</h1>
      <dl>
      </dl>
      "
      .unindent()
    );
  }
}
