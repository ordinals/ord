use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneBalancesHtml {
  pub balances: BTreeMap<Rune, BTreeMap<OutPoint, Pile>>,
}

impl PageContent for RuneBalancesHtml {
  fn title(&self) -> String {
    "Rune Balances".to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const RUNE: u128 = 99246114928149462;

  #[test]
  fn display_rune_balances() {
    let balances: BTreeMap<Rune, BTreeMap<OutPoint, Pile>> = vec![
      (
        Rune(RUNE),
        vec![(
          OutPoint {
            txid: txid(1),
            vout: 1,
          },
          Pile {
            amount: 1000,
            divisibility: 0,
            symbol: Some('$'),
          },
        )]
        .into_iter()
        .collect(),
      ),
      (
        Rune(RUNE + 1),
        vec![(
          OutPoint {
            txid: txid(2),
            vout: 2,
          },
          Pile {
            amount: 12345678,
            divisibility: 1,
            symbol: Some('¢'),
          },
        )]
        .into_iter()
        .collect(),
      ),
    ]
    .into_iter()
    .collect();

    assert_regex_match!(
      RuneBalancesHtml { balances }.to_string(),
      "<h1>Rune Balances</h1>
<table>
  <tr>
    <th>rune</th>
    <th>balances</th>
  </tr>
  <tr>
    <td><a href=/rune/AAAAAAAAAAAAA>.*</a></td>
    <td>
      <table>
        <tr>
          <td class=monospace>
            <a href=/output/1{64}:1>1{64}:1</a>
          </td>
          <td class=monospace>
            1000\u{A0}\\$
          </td>
        </tr>
      </table>
    </td>
  </tr>
  <tr>
    <td><a href=/rune/AAAAAAAAAAAAB>.*</a></td>
    <td>
      <table>
        <tr>
          <td class=monospace>
            <a href=/output/2{64}:2>2{64}:2</a>
          </td>
          <td class=monospace>
            1234567\\.8\u{A0}¢
          </td>
        </tr>
      </table>
    </td>
  </tr>
</table>
"
      .unindent()
    );
  }
}
