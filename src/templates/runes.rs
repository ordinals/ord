use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunesHtml {
  pub entries: Vec<(RuneId, RuneEntry)>,
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
          RuneId { block: 0, tx: 0 },
          RuneEntry {
            spaced_rune: SpacedRune {
              rune: Rune(26),
              spacers: 1
            },
            ..default()
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
