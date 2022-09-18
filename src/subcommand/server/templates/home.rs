use super::*;

#[derive(Boilerplate)]
pub(crate) struct HomeHtml {
  last: u64,
  blocks: Vec<(Rarity, BlockHash)>,
}

impl HomeHtml {
  pub(crate) fn new(blocks: Vec<(u64, BlockHash)>) -> Self {
    Self {
      last: blocks
        .get(0)
        .map(|(height, _)| height)
        .cloned()
        .unwrap_or(0),
      blocks: blocks
        .into_iter()
        .map(|(height, hash)| (Height(height).starting_ordinal().rarity(), hash))
        .collect(),
    }
  }
}

impl Content for HomeHtml {
  fn title(&self) -> String {
    "Ordinals".to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn home_html() {
    assert_regex_match!(
      &HomeHtml::new(vec![
        (
          1,
          "1111111111111111111111111111111111111111111111111111111111111111"
            .parse()
            .unwrap()
        ),
        (
          0,
          "0000000000000000000000000000000000000000000000000000000000000000"
            .parse()
            .unwrap()
        )
      ],)
      .to_string(),
      "<h1>Ordinals</h1>
<nav>.*</nav>
<h2>Search</h2>
<form action=/search method=get>
  <input type=text name=query>
  <input type=submit>
</form>
<h2>Recent Blocks</h2>
<ol start=1 reversed class=monospace>
  <li><a href=/block/1{64} class=uncommon>1{64}</a></li>
  <li><a href=/block/0{64} class=mythic>0{64}</a></li>
</ol>
",
    );
  }
}
