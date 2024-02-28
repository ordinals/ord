use super::*;

#[derive(Deserialize, Default, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
  pub(crate) bitcoin_rpc_pass: Option<String>,
  pub(crate) bitcoin_rpc_user: Option<String>,
  pub(crate) chain: Option<Chain>,
  pub(crate) hidden: Option<HashSet<InscriptionId>>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn example_config_file_is_valid() {
    let _: Config = serde_yaml::from_reader(File::open("ord.yaml").unwrap()).unwrap();
  }
}
