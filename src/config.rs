use super::*;

#[derive(Deserialize, Default, PartialEq, Debug)]
pub(crate) struct Config {
  pub(crate) hidden: HashSet<InscriptionId>,
}

impl Config {
  pub(crate) fn is_hidden(&self, inscription_id: InscriptionId) -> bool {
    self.hidden.contains(&inscription_id)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inscriptions_can_be_hidden() {
    let a = "8d363b28528b0cb86b5fd48615493fb175bdf132d2a3d20b4251bba3f130a5abi0"
      .parse::<InscriptionId>()
      .unwrap();

    let b = "8d363b28528b0cb86b5fd48615493fb175bdf132d2a3d20b4251bba3f130a5abi1"
      .parse::<InscriptionId>()
      .unwrap();

    let config = Config {
      hidden: iter::once(a).collect(),
    };

    assert!(config.is_hidden(a));
    assert!(!config.is_hidden(b));
  }
}
