use {super::*, crate::subcommand::wallet::inscribe::mode::Mode};

#[derive(Deserialize, Default, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchEntry {
  inscription: PathBuf,
  metadata: Option<PathBuf>,
  metaprotocol: Option<String>,
}

#[derive(Deserialize, Default, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchConfig {
  batch: Vec<BatchEntry>,
  parent: Option<InscriptionId>,
  mode: Mode,
}

#[derive(Serialize, Deserialize)]
pub struct BatchOutput {
  pub outputs: Vec<inscribe::Output>,
}

#[derive(Debug, Parser)]
pub(crate) struct BatchInscribe {
  #[arg(long, help = "Don't sign or broadcast transactions.")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB.")]
  pub(crate) fee_rate: FeeRate,
  #[arg(help = "Read YAML batch <FILE> that specifies all inscription info.")]
  pub(crate) file: PathBuf,
}

impl BatchInscribe {
  pub(crate) fn run(self, _options: Options) -> SubcommandResult {
    let batch = self.load_batch()?;

    let _batch = batch.batch;

    Ok(Box::new(BatchOutput {
      outputs: Vec::new(),
    }))
  }

  pub(crate) fn load_batch(&self) -> Result<BatchConfig> {
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
        "mode: shared-output\nparent: {parent}\nbatch:\n- inscription: {}\n  metadata: {}\n- inscription: {}\n  metaprotocol: brc-20\n",
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
        "--fee-rate",
        "2.1"
      ])
      .unwrap()
      .subcommand
      {
        Subcommand::Wallet(wallet::Wallet::BatchInscribe(batch_inscribe)) =>
          batch_inscribe.load_batch().unwrap(),
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
      "--fee-rate",
      "2.1"
    ])
    .unwrap()
    .subcommand
    {
      Subcommand::Wallet(wallet::Wallet::BatchInscribe(batch_inscribe)) =>
        batch_inscribe.load_batch().is_err(),
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
      "--fee-rate",
      "2.1"
    ])
    .unwrap()
    .subcommand
    {
      Subcommand::Wallet(wallet::Wallet::BatchInscribe(batch_inscribe)) =>
        batch_inscribe.load_batch().is_err(),
      _ => panic!(),
    })
  }
}
