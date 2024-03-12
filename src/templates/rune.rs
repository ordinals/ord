use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneHtml {
  pub entry: RuneEntry,
  pub id: RuneId,
  pub parent: Option<InscriptionId>,
}

impl PageContent for RuneHtml {
  fn title(&self) -> String {
    format!("Rune {}", self.entry.spaced_rune())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, crate::runes::Rune};

  #[test]
  fn display() {
    assert_regex_match!(
      RuneHtml {
        entry: RuneEntry {
          burned: 123456789123456789,
          divisibility: 9,
          etching: Txid::all_zeros(),
          mints: 100,
          number: 25,
          mint: Some(MintEntry {
            end: Some(11),
            limit: Some(1000000001),
            deadline: Some(7),
          }),
          rune: Rune(u128::MAX),
          spacers: 1,
          supply: 123456789123456789,
          symbol: Some('%'),
          timestamp: 0,
        },
        id: RuneId {
          height: 10,
          index: 9,
        },
        parent: Some(InscriptionId {
          txid: Txid::all_zeros(),
          index: 0,
        }),
      },
      "<h1>B•CGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
<iframe .* src=/preview/0{64}i0></iframe>
<dl>
  <dt>number</dt>
  <dd>25</dd>
  <dt>timestamp</dt>
  <dd><time>1970-01-01 00:00:00 UTC</time></dd>
  <dt>id</dt>
  <dd>10:9</dd>
  <dt>etching block height</dt>
  <dd><a href=/block/10>10</a></dd>
  <dt>etching transaction index</dt>
  <dd>9</dd>
  <dt>mint</dt>
  <dd>
    <dl>
      <dt>deadline</dt>
      <dd><time>1970-01-01 00:00:07 UTC</time></dd>
      <dt>end</dt>
      <dd><a href=/block/11>11</a></dd>
      <dt>limit</dt>
      <dd>1.000000001 %</dd>
      <dt>mints</dt>
      <dd>100</dd>
    </dl>
  </dd>
  <dt>supply</dt>
  <dd>123456789.123456789\u{00A0}%</dd>
  <dt>burned</dt>
  <dd>123456789.123456789\u{00A0}%</dd>
  <dt>divisibility</dt>
  <dd>9</dd>
  <dt>symbol</dt>
  <dd>%</dd>
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
          burned: 123456789123456789,
          mint: None,
          divisibility: 9,
          etching: Txid::all_zeros(),
          mints: 0,
          number: 25,
          rune: Rune(u128::MAX),
          spacers: 1,
          supply: 123456789123456789,
          symbol: Some('%'),
          timestamp: 0,
        },
        id: RuneId {
          height: 10,
          index: 9,
        },
        parent: None,
      },
      "<h1>B•CGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
<dl>
  <dt>number</dt>
  <dd>25</dd>
  <dt>timestamp</dt>
  <dd><time>1970-01-01 00:00:00 UTC</time></dd>
  <dt>id</dt>
  <dd>10:9</dd>
  <dt>etching block height</dt>
  <dd><a href=/block/10>10</a></dd>
  <dt>etching transaction index</dt>
  <dd>9</dd>
  <dt>mint</dt>
  <dd>no</dd>
  <dt>supply</dt>
  <dd>123456789.123456789\u{00A0}%</dd>
  <dt>burned</dt>
  <dd>123456789.123456789\u{00A0}%</dd>
  <dt>divisibility</dt>
  <dd>9</dd>
  <dt>symbol</dt>
  <dd>%</dd>
  <dt>etching</dt>
  <dd><a class=monospace href=/tx/0{64}>0{64}</a></dd>
</dl>
"
    );
  }

  #[test]
  fn display_empty_mint() {
    assert_regex_match!(
      RuneHtml {
        entry: RuneEntry {
          burned: 123456789123456789,
          mint: Some(MintEntry {
            deadline: None,
            end: None,
            limit: None,
          }),
          divisibility: 9,
          etching: Txid::all_zeros(),
          mints: 0,
          number: 25,
          rune: Rune(u128::MAX),
          spacers: 1,
          supply: 123456789123456789,
          symbol: Some('%'),
          timestamp: 0,
        },
        id: RuneId {
          height: 10,
          index: 9,
        },
        parent: None,
      },
      "<h1>B•CGDENLQRQWDSLRUGSNLBTMFIJAV</h1>
<dl>
  <dt>number</dt>
  <dd>25</dd>
  <dt>timestamp</dt>
  <dd><time>1970-01-01 00:00:00 UTC</time></dd>
  <dt>id</dt>
  <dd>10:9</dd>
  <dt>etching block height</dt>
  <dd><a href=/block/10>10</a></dd>
  <dt>etching transaction index</dt>
  <dd>9</dd>
  <dt>mint</dt>
  <dd>
    <dl>
      <dt>deadline</dt>
      <dd>none</dd>
      <dt>end</dt>
      <dd>none</dd>
      <dt>limit</dt>
      <dd>none</dd>
      <dt>mints</dt>
      <dd>0</dd>
    </dl>
  </dd>
  <dt>supply</dt>
  <dd>123456789.123456789\u{00A0}%</dd>
  <dt>burned</dt>
  <dd>123456789.123456789\u{00A0}%</dd>
  <dt>divisibility</dt>
  <dd>9</dd>
  <dt>symbol</dt>
  <dd>%</dd>
  <dt>etching</dt>
  <dd><a class=monospace href=/tx/0{64}>0{64}</a></dd>
</dl>
"
    );
  }
}
