use super::*;

#[derive(Display)]
pub(crate) struct OutputHtml {
  pub(crate) outpoint: OutPoint,
  pub(crate) list: List,
}

impl Content for OutputHtml {
  fn title(&self) -> String {
    format!("Output {}", self.outpoint)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, unindent::Unindent};

  #[test]
  fn unspent_output() {
    assert_eq!(
      OutputHtml {
        outpoint: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0"
          .parse()
          .unwrap(),
        list: List::Unspent(vec![(0, 1), (1, 2)])
      }
      .to_string(),
      "
        <h1>Output 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0</h1>
        <h2>Ordinal Ranges</h2>
        <ul>
          <li><a href=/range/0/1 class=mythic>[0,1)</a></li>
          <li><a href=/range/1/2 class=common>[1,2)</a></li>
        </ul>
      "
      .unindent()
    );
  }

  #[test]
  fn spent_output() {
    assert_eq!(
      OutputHtml {
        outpoint: "0000000000000000000000000000000000000000000000000000000000000000:0"
          .parse()
          .unwrap(),
        list: List::Spent("1111111111111111111111111111111111111111111111111111111111111111".parse().unwrap())
      }
      .to_string(),
      "
        <h1>Output 0000000000000000000000000000000000000000000000000000000000000000:0</h1>
        <p>Spent by transaction <a href=/tx/1111111111111111111111111111111111111111111111111111111111111111>1111111111111111111111111111111111111111111111111111111111111111</a>.</p>
      "
      .unindent()
    );
  }
}
