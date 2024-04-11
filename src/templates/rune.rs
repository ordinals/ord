use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneHtml {
  pub entry: RuneEntry,
  pub id: RuneId,
  pub mintable: bool,
  pub parent: Option<InscriptionId>,
}

impl PageContent for RuneHtml {
  fn title(&self) -> String {
    format!("Rune {}", self.entry.spaced_rune)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_regex_match!(
      RuneHtml {
        entry: RuneEntry {
          block: 1,
          burned: 123456789123456789,
          divisibility: 9,
          etching: Txid::all_zeros(),
          mints: 100,
          terms: Some(Terms {
            cap: Some(101),
            offset: (None, None),
            height: (Some(10), Some(11)),
            amount: Some(1000000001),
          }),
          number: 25,
          premine: 123456789,
          spaced_rune: SpacedRune {
            rune: Rune(u128::MAX),
            spacers: 1
          },
          symbol: Some('%'),
          timestamp: 0,
          turbo: true,
        },
        id: RuneId { block: 10, tx: 9 },
        mintable: true,
        parent: Some(InscriptionId {
          txid: Txid::all_zeros(),
          index: 0,
        }),
      },
      "<h1>B•CGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
.*<a href=/inscription/.*<iframe .* src=/preview/0{64}i0></iframe></a>.*
<dl>
  <dt>number</dt>
  <dd>25</dd>
  <dt>timestamp</dt>
  <dd><time>1970-01-01 00:00:00 UTC</time></dd>
  <dt>id</dt>
  <dd>10:9</dd>
  <dt>etching block</dt>
  <dd><a href=/block/10>10</a></dd>
  <dt>etching transaction</dt>
  <dd>9</dd>
  <dt>mint</dt>
  <dd>
    <dl>
      <dt>start</dt>
      <dd><a href=/block/10>10</a></dd>
      <dt>end</dt>
      <dd><a href=/block/11>11</a></dd>
      <dt>amount</dt>
      <dd>1.000000001 %</dd>
      <dt>mints</dt>
      <dd>100</dd>
      <dt>cap</dt>
      <dd>101</dd>
      <dt>remaining</dt>
      <dd>1</dd>
      <dt>mintable</dt>
      <dd>true</dd>
    </dl>
  </dd>
  <dt>supply</dt>
  <dd>100.123456889\u{A0}%</dd>
  <dt>premine</dt>
  <dd>0.123456789\u{A0}%</dd>
  <dt>burned</dt>
  <dd>123456789.123456789\u{A0}%</dd>
  <dt>divisibility</dt>
  <dd>9</dd>
  <dt>symbol</dt>
  <dd>%</dd>
  <dt>turbo</dt>
  <dd>true</dd>
  <dt>etching</dt>
  <dd><a class=monospace href=/tx/0{64}>0{64}</a></dd>
  <dt>parent</dt>
  <dd><a class=monospace href=/inscription/0{64}i0>0{64}i0</a></dd>
</dl>
"
    );
  }

  #[test]
  fn display_no_mint() {
    assert_regex_match!(
      RuneHtml {
        entry: RuneEntry {
          block: 0,
          burned: 123456789123456789,
          terms: None,
          divisibility: 9,
          etching: Txid::all_zeros(),
          mints: 0,
          number: 25,
          premine: 0,
          spaced_rune: SpacedRune {
            rune: Rune(u128::MAX),
            spacers: 1
          },
          symbol: Some('%'),
          timestamp: 0,
          turbo: false,
        },
        id: RuneId { block: 10, tx: 9 },
        mintable: false,
        parent: None,
      },
      "<h1>B•CGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
<dl>.*
  <dt>mint</dt>
  <dd>no</dd>
.*</dl>
"
    );
  }

  #[test]
  fn display_no_turbo() {
    assert_regex_match!(
      RuneHtml {
        entry: RuneEntry {
          block: 0,
          burned: 123456789123456789,
          terms: None,
          divisibility: 9,
          etching: Txid::all_zeros(),
          mints: 0,
          number: 25,
          premine: 0,
          spaced_rune: SpacedRune {
            rune: Rune(u128::MAX),
            spacers: 1
          },
          symbol: Some('%'),
          timestamp: 0,
          turbo: false,
        },
        id: RuneId { block: 10, tx: 9 },
        mintable: false,
        parent: None,
      },
      "<h1>B•CGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
<dl>.*
  <dt>turbo</dt>
  <dd>false</dd>
.*</dl>
"
    );
  }

  #[test]
  fn display_empty_mint() {
    assert_regex_match!(
      RuneHtml {
        entry: RuneEntry {
          block: 0,
          burned: 123456789123456789,
          terms: Some(Terms {
            cap: None,
            offset: (None, None),
            height: (None, None),
            amount: None,
          }),
          divisibility: 9,
          etching: Txid::all_zeros(),
          mints: 0,
          premine: 0,
          number: 25,
          spaced_rune: SpacedRune {
            rune: Rune(u128::MAX),
            spacers: 1
          },
          symbol: Some('%'),
          timestamp: 0,
          turbo: false,
        },
        id: RuneId { block: 10, tx: 9 },
        mintable: false,
        parent: None,
      },
      "<h1>B•CGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
<dl>.*
  <dt>mint</dt>
  <dd>
    <dl>
      <dt>start</dt>
      <dd>none</dd>
      <dt>end</dt>
      <dd>none</dd>
      <dt>amount</dt>
      <dd>none</dd>
      <dt>mints</dt>
      <dd>0</dd>
      <dt>cap</dt>
      <dd>0</dd>
      <dt>remaining</dt>
      <dd>0</dd>
      <dt>mintable</dt>
      <dd>false</dd>
    </dl>
  </dd>
.*</dl>
"
    );
  }
}
