use super::*;

#[derive(Boilerplate)]
pub(crate) struct RunesHtml {
  pub(crate) entries: Vec<(RuneId, RuneEntry)>,
}

impl PageContent for RunesHtml {
  fn title(&self) -> String {
    "Runes".to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(
      RunesHtml {
        entries: vec![(
          RuneId {
            height: 0,
            index: 0,
          },
          RuneEntry {
            rune: Rune(26),
            spacers: 1,
            ..Default::default()
          }
        )],
      }
      .to_string(),
      "<h1>Runes</h1>
<ul>
  <li><a href=/rune/A•A>A•A</a></li>
</ul>
"
    );
  }
}
