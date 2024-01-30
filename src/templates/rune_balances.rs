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
