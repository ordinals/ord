use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Annex {
  #[arg(long, help = "Compress inscription content with brotli.")]
  pub(crate) compress: bool,
  #[arg(
    long,
    help = "Create annex with multiple inscriptions as defined in YAML <BATCH_FILE>.",
    value_name = "BATCH_FILE"
  )]
  pub(crate) batch: PathBuf,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub annex: String,
}

impl Annex {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let utxos = wallet.utxos();

    let batchfile = batch::File::load(&self.batch)?;

    for inscription in &batchfile.inscriptions {
      for inscription_id in &inscription.gallery {
        ensure! {
          wallet.inscription_exists(*inscription_id)?,
          "gallery item does not exist: {inscription_id}",
        }
      }
    }

    let parent_info = wallet.get_parent_info(&batchfile.parents)?;

    let (inscriptions, _, _, _) = batchfile.inscriptions(
      &wallet,
      utxos,
      parent_info
        .iter()
        .map(|info| info.tx_out.value.to_sat())
        .collect(),
      self.compress,
    )?;

    let annex = hex::encode(Inscription::convert_batch_to_annex(&inscriptions));

    Ok(Some(Box::new(annex::Output { annex })))
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    crate::wallet::batch,
    serde_yaml::{Mapping, Value},
    tempfile::TempDir,
  };

  #[test]
  fn batch_is_loaded_from_yaml_file() {
    let parent = "8d363b28528b0cb86b5fd48615493fb175bdf132d2a3d20b4251bba3f130a5abi0"
      .parse::<InscriptionId>()
      .unwrap();

    let tempdir = TempDir::new().unwrap();

    let inscription_path = tempdir.path().join("tulip.txt");
    fs::write(&inscription_path, "tulips are pretty").unwrap();

    let brc20_path = tempdir.path().join("token.json");

    let batch_path = tempdir.path().join("batch.yaml");
    fs::write(
      &batch_path,
      format!(
        "mode: separate-outputs
parents:
- {parent}
inscriptions:
- file: {}
  metadata:
    title: Lorem Ipsum
    description: Lorem ipsum dolor sit amet, consectetur adipiscing elit. In tristique, massa nec condimentum venenatis, ante massa tempor velit, et accumsan ipsum ligula a massa. Nunc quis orci ante.
- file: {}
  metaprotocol: brc-20
",
        inscription_path.display(),
        brc20_path.display()
      ),
    )
    .unwrap();

    let mut metadata = Mapping::new();
    metadata.insert(
      Value::String("title".to_string()),
      Value::String("Lorem Ipsum".to_string()),
    );
    metadata.insert(Value::String("description".to_string()), Value::String("Lorem ipsum dolor sit amet, consectetur adipiscing elit. In tristique, massa nec condimentum venenatis, ante massa tempor velit, et accumsan ipsum ligula a massa. Nunc quis orci ante.".to_string()));

    assert_eq!(
      batch::File::load(&batch_path).unwrap(),
      batch::File {
        inscriptions: vec![
          batch::Entry {
            file: Some(inscription_path),
            metadata: Some(Value::Mapping(metadata)),
            ..default()
          },
          batch::Entry {
            file: Some(brc20_path),
            metaprotocol: Some("brc-20".to_string()),
            ..default()
          }
        ],
        parents: vec![parent],
        ..default()
      }
    );
  }

  #[test]
  fn batch_with_unknown_field_throws_error() {
    let tempdir = TempDir::new().unwrap();
    let batch_path = tempdir.path().join("batch.yaml");
    fs::write(
      &batch_path,
      "mode: shared-output\ninscriptions:\n- file: meow.wav\nunknown: 1.)what",
    )
    .unwrap();

    assert!(batch::File::load(&batch_path)
      .unwrap_err()
      .to_string()
      .contains("unknown field `unknown`"));
  }
}
