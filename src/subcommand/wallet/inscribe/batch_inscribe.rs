use {super::mode::Mode, super::*};

#[derive(Deserialize, Default, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchEntry {
  inscription: PathBuf,
  metadata: Option<PathBuf>,
  metaprotocol: Option<String>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchConfig {
  batch: Vec<BatchEntry>,
  dry_run: bool,
  fee_rate: FeeRate,
  parent: Option<InscriptionId>,
  mode: Mode,
}

#[derive(Serialize, Deserialize)]
pub struct BatchOutput {
  pub outputs: Vec<inscribe::Output>,
}

#[derive(Debug, Parser)]
pub(crate) struct BatchInscribe {
  #[arg(help = "Read YAML batch <FILE> that specifies all inscription info.")]
  pub(crate) file: PathBuf,
}

impl BatchInscribe {
  pub(crate) fn run(self, _options: Options) -> SubcommandResult {
    let batch_config = self.load_batch_config()?;



    Ok(Box::new(BatchOutput {
      outputs: Vec::new(),
    }))
  }

  pub(crate) fn load_batch_config(&self) -> Result<BatchConfig> {
    Ok(serde_yaml::from_reader(File::open(self.file.clone())?)?)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn batch_is_loaded_from_yaml_file() {
    let parent = "8d363b28528b0cb86b5fd48615493fb175bdf132d2a3d20b4251bba3f130a5abi0"
      .parse::<InscriptionId>()
      .unwrap();

    let tempdir = TempDir::new().unwrap();

    let inscription_path = tempdir.path().join("tulip.txt");
    let metadata_path = tempdir.path().join("metadata.json");
    fs::write(&inscription_path, "tulips are pretty").unwrap();

    let brc20_path = tempdir.path().join("token.json");

    let batch_path = tempdir.path().join("batch.yaml");
    fs::write(
      &batch_path,
      format!(
        "dry_run: false\nfee_rate: 2.1\nmode: shared-output\nparent: {parent}\nbatch:\n- inscription: {}\n  metadata: {}\n- inscription: {}\n  metaprotocol: brc-20\n",
        inscription_path.display(),
        metadata_path.display(),
        brc20_path.display()
      ),
    )
    .unwrap();

    pretty_assert_eq!(
      match Arguments::try_parse_from([
        "ord",
        "wallet",
        "batch-inscribe",
        batch_path.to_str().unwrap(),
      ])
      .unwrap()
      .subcommand
      {
        Subcommand::Wallet(wallet::Wallet::BatchInscribe(batch_inscribe)) =>
          batch_inscribe.load_batch_config().unwrap(),
        _ => panic!(),
      },
      BatchConfig {
        batch: vec![
          BatchEntry {
            inscription: inscription_path,
            metadata: Some(metadata_path),
            ..Default::default()
          },
          BatchEntry {
            inscription: brc20_path,
            metaprotocol: Some("brc-20".to_string()),
            ..Default::default()
          }
        ],
        dry_run: false,
        fee_rate: FeeRate::try_from(2.1).unwrap(),
        parent: Some(parent),
        mode: Mode::SharedOutput,
      }
    );
  }

  #[test]
  fn batch_with_invalid_field_value_throws_error() {
    let tempdir = TempDir::new().unwrap();

    let inscription_path = tempdir.path().join("tulip.txt");
    fs::write(&inscription_path, "tulips are pretty").unwrap();

    let batch_path = tempdir.path().join("batch.yaml");
    fs::write(
      &batch_path,
      format!(
        "mode: wrong-mode\nbatch:\n- inscription: {}\n",
        inscription_path.display(),
      ),
    )
    .unwrap();

    assert!(match Arguments::try_parse_from([
      "ord",
      "wallet",
      "batch-inscribe",
      batch_path.to_str().unwrap(),
    ])
    .unwrap()
    .subcommand
    {
      Subcommand::Wallet(wallet::Wallet::BatchInscribe(batch_inscribe)) =>
        batch_inscribe.load_batch_config().is_err(),
      _ => panic!(),
    })
  }

  #[test]
  fn batch_is_unknown_field_throws_error() {
    let tempdir = TempDir::new().unwrap();
    let inscription_path = tempdir.path().join("tulip.txt");
    fs::write(&inscription_path, "tulips are pretty").unwrap();

    let batch_path = tempdir.path().join("batch.yaml");
    fs::write(
      &batch_path,
      format!(
        "mode: shared-output\nbatch:\n- inscription: {}\nunknown: 1.)what",
        inscription_path.display(),
      ),
    )
    .unwrap();

    assert!(match Arguments::try_parse_from([
      "ord",
      "wallet",
      "batch-inscribe",
      batch_path.to_str().unwrap(),
    ])
    .unwrap()
    .subcommand
    {
      Subcommand::Wallet(wallet::Wallet::BatchInscribe(batch_inscribe)) =>
        batch_inscribe.load_batch_config().is_err(),
      _ => panic!(),
    })
  }
}
