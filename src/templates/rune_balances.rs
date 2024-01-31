use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneBalancesHtml {
  pub balances: BTreeMap<Rune, BTreeMap<OutPoint, u128>>,
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
    let mut balances: BTreeMap<Rune, BTreeMap<OutPoint, u128>> = BTreeMap::new();

    let rune_a = Rune(RUNE);
    let rune_b = Rune(RUNE + 1);

    let mut rune_a_balances: BTreeMap<OutPoint, u128> = BTreeMap::new();
    rune_a_balances.insert(
      OutPoint {
        txid: txid(1),
        vout: 1,
      },
      1000,
    );

    let mut rune_b_balances: BTreeMap<OutPoint, u128> = BTreeMap::new();
    rune_b_balances.insert(
      OutPoint {
        txid: txid(2),
        vout: 2,
      },
      12345678,
    );

    balances.insert(rune_a, rune_a_balances);
    balances.insert(rune_b, rune_b_balances);

    assert_regex_match!(
      RuneBalancesHtml { balances }.to_string(),
      "<h1>Rune Balances</h1>
<table class=full-width-table>
  <tr>
    <th>rune</th>
    <th>balances</th>
  </tr>
  <tr>
    <td class=center><a href=/rune/AAAAAAAAAAAAA>.*</a></td>
    <td>
      <table class=full-width>
        <tr>
          <td class=monospace>
            <a href=/output/1111111111111111111111111111111111111111111111111111111111111111:1>.*</a>
          </td>
          <td class=\"monospace right-align\">
            1000
          </td>
        </tr>
      </table>
    </td>
  </tr>
  <tr>
    <td class=center><a href=/rune/AAAAAAAAAAAAB>.*</a></td>
    <td>
      <table class=full-width>
        <tr>
          <td class=monospace>
            <a href=/output/2222222222222222222222222222222222222222222222222222222222222222:2>.*</a>
          </td>
          <td class=\"monospace right-align\">
            12345678
          </td>
        </tr>
      </table>
    </td>
  </tr>
</table>
".unindent());
  }
}
