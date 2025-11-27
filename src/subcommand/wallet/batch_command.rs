use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Batch {
  #[command(flatten)]
  shared: SharedArgs,
  #[arg(
    long,
    help = "Inscribe multiple inscriptions defined in YAML <BATCH_FILE>.",
    value_name = "BATCH_FILE"
  )]
  pub(crate) batch: PathBuf,
}

impl Batch {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let utxos = wallet.utxos();

    let batchfile = batch::File::load(&self.batch)?;

    for inscription in &batchfile.inscriptions {
      for item in &inscription.gallery {
        ensure! {
          wallet.inscription_exists(item.id)?,
          "gallery item does not exist: {}", item.id,
        }
      }
    }

    let parent_info = wallet.get_parent_info(&batchfile.parents)?;

    let (inscriptions, reveal_satpoints, postages, destinations) = batchfile.inscriptions(
      &wallet,
      utxos,
      parent_info
        .iter()
        .map(|info| info.tx_out.value.to_sat())
        .collect(),
      self.shared.compress,
    )?;

    let mut locked_utxos = wallet.locked_utxos().clone();

    locked_utxos.extend(
      reveal_satpoints
        .iter()
        .map(|(satpoint, txout)| (satpoint.outpoint, txout.clone())),
    );

    batch::Plan {
      commit_fee_rate: self.shared.commit_fee_rate.unwrap_or(self.shared.fee_rate),
      destinations,
      dry_run: self.shared.dry_run,
      inscriptions,
      mode: batchfile.mode,
      no_backup: self.shared.no_backup,
      no_limit: self.shared.no_limit,
      parent_info,
      postages,
      reinscribe: batchfile.reinscribe,
      reveal_fee_rate: self.shared.fee_rate,
      reveal_satpoints,
      satpoint: if let Some(sat) = batchfile.sat {
        Some(wallet.find_sat_in_outputs(sat)?)
      } else {
        batchfile.satpoint
      },
    }
    .inscribe(&locked_utxos.into_keys().collect(), utxos, &wallet)
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

    assert!(
      batch::File::load(&batch_path)
        .unwrap_err()
        .to_string()
        .contains("unknown field `unknown`")
    );
  }
}
