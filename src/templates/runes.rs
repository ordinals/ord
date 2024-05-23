use super::*;

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunesHtml {
  pub entries: Vec<(RuneId, RuneEntry)>,
  pub more: bool,
  pub prev: Option<usize>,
  pub next: Option<usize>,
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
        more: false,
        prev: None,
        next: None,
      }
      .to_string(),
      "<h1>Runes</h1>
<ul>
  <li><a href=/rune/A•A>A•A</a></li>
</ul>
<div class=center>
    prev
      next
  </div>"
    );
  }

  #[test]
  fn with_prev_and_next() {
    assert_eq!(
      RunesHtml {
        entries: vec![
          (
            RuneId { block: 0, tx: 0 },
            RuneEntry {
              spaced_rune: SpacedRune {
                rune: Rune(0),
                spacers: 0
              },
              ..Default::default()
            }
          ),
          (
            RuneId { block: 0, tx: 1 },
            RuneEntry {
              spaced_rune: SpacedRune {
                rune: Rune(2),
                spacers: 0
              },
              ..Default::default()
            }
          )
        ],
        prev: Some(1),
        next: Some(2),
        more: true,
      }
      .to_string(),
      "<h1>Runes</h1>
<ul>
  <li><a href=/rune/A>A</a></li>
  <li><a href=/rune/C>C</a></li>
</ul>
<div class=center>
    <a class=prev href=/runes/1>prev</a>
      <a class=next href=/runes/2>next</a>
  </div>"
    );
  }
}
