use super::*;

#[derive(Deserialize, Default, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchEntry {
  destination: Option<Address<NetworkUnchecked>>,
  inscription: PathBuf,
  metadata: Option<PathBuf>,
  metaprotocol: Option<String>,
  parent: Option<InscriptionId>,
}

#[derive(Deserialize, Default, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct BatchConfig {
  batch: Vec<BatchEntry>,
  // mode: Option<Mode>,
}

//pub(crate) enum Mode {
//  SharedOutput,
//  SeperateOutputs,
//}
//
//impl fmt::Display for Mode {
//  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//    write!(
//      f,
//      "{}",
//      match self {
//        Mode::SharedOutput => "shared-output",
//        Mode::SeperateOutputs => "seperate-outputs",
//      }
//    )
//  }
//}
//
//impl Default for Mode {
//  fn default() -> Self {
//    Mode::SeperateOutputs
//  }
//}

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
    let batch_path = tempdir.path().join("batch.yaml");
    let inscription_path = tempdir.path().join("tulip.txt");

    fs::write(
      &batch_path,
      format!(
        "batch:\n- inscription: {}\n  parent: {parent}\n",
        inscription_path.display()
      ),
    )
    .unwrap();

    fs::write(&inscription_path, "tulips are pretty").unwrap();

    assert_eq!(
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
        batch: vec![BatchEntry {
          inscription: inscription_path,
          parent: Some(parent),
          ..Default::default()
        }],
      }
    );
  }
}
