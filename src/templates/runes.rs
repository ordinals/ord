use {super::*, crate::index::entry::RuneInfo};

#[derive(Boilerplate, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunesHtml {
  pub runes: Vec<RuneInfo>,
  pub more: bool,
  pub prev: Option<u64>,
  pub next: Option<u64>,
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
        runes: vec![RuneInfo {
          id: RuneId {
            height: 0,
            index: 0,
          },
          entry: RuneEntry {
            rune: Rune(26),
            spacers: 1,
            ..Default::default()
          }
        }],
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
  </div>
"
    );
  }

  #[test]
  fn with_prev_and_next() {
    assert_eq!(
      RunesHtml {
        runes: vec![
          RuneInfo {
            id: RuneId {
              height: 0,
              index: 0
            },
            entry: RuneEntry {
              rune: Rune(0),
              ..Default::default()
            }
          },
          RuneInfo {
            id: RuneId {
              height: 0,
              index: 0
            },
            entry: RuneEntry {
              rune: Rune(2),
              ..Default::default()
            }
          }
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
  </div>
"
    );
  }
}
