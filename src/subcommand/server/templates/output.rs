use super::*;

#[derive(Display)]
pub(crate) struct OutputHtml {
  pub(crate) outpoint: OutPoint,
  pub(crate) ranges: Vec<(u64, u64)>,
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
  fn output_html() {
    assert_eq!(
      OutputHtml {
        outpoint: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0"
          .parse()
          .unwrap(),
        ranges: vec![(0, 1), (1, 2)]
      }
      .to_string(),
      "
        <h1>Output 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0</h1>
        <h2>Ordinal Ranges</h2>
        <ul>
          <li><a href=/range/0/1>[0,1)</a></li>
          <li><a href=/range/1/2>[1,2)</a></li>
        </ul>
      "
      .unindent()
    );
  }
}
