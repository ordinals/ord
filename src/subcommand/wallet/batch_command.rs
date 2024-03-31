use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Batch {
  #[command(flatten)]
  shared: SharedArgs,
  #[arg(
    long,
    help = "Inscribe multiple inscriptions and rune defined in YAML <BATCH_FILE>."
  )]
  pub(crate) batch: PathBuf,
}

impl Batch {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let utxos = wallet.utxos();

    let batchfile = batch::File::load(&self.batch)?;

    let parent_info = wallet.get_parent_info(batchfile.parent)?;

    let (inscriptions, reveal_satpoints, postages, destinations) = batchfile.inscriptions(
      &wallet,
      utxos,
      parent_info.as_ref().map(|info| info.tx_out.value),
      self.shared.compress,
    )?;

    let mut locked_utxos = wallet.locked_utxos().clone();

    locked_utxos.extend(
      reveal_satpoints
        .iter()
        .map(|(satpoint, txout)| (satpoint.outpoint, txout.clone())),
    );

    if let Some(etching) = batchfile.etching {
      Self::check_etching(&wallet, &etching)?;
    }

    batch::Plan {
      commit_fee_rate: self.shared.commit_fee_rate.unwrap_or(self.shared.fee_rate),
      destinations,
      dry_run: self.shared.dry_run,
      etching: batchfile.etching,
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
    .inscribe(
      &locked_utxos.into_keys().collect(),
      wallet.get_runic_outputs()?,
      utxos,
      &wallet,
    )
  }

  fn check_etching(wallet: &Wallet, etching: &batch::Etching) -> Result {
    let rune = etching.rune.rune;

    ensure!(
      wallet.load_etching(rune)?.is_none(),
      "rune `{rune}` has pending etching, resume with `ord wallet resume`"
    );

    ensure!(!rune.is_reserved(), "rune `{rune}` is reserved");

    ensure!(
      etching.divisibility <= Etching::MAX_DIVISIBILITY,
      "<DIVISIBILITY> must be less than or equal 38"
    );

    ensure!(
      wallet.has_rune_index(),
      "etching runes requires index created with `--index-runes`",
    );

    ensure!(
      wallet.get_rune(rune)?.is_none(),
      "rune `{rune}` has already been etched",
    );

    let premine = etching.premine.to_integer(etching.divisibility)?;

    let supply = etching.supply.to_integer(etching.divisibility)?;

    let mintable = etching
      .terms
      .map(|terms| -> Result<u128> {
        terms
          .cap
          .checked_mul(terms.amount.to_integer(etching.divisibility)?)
          .ok_or_else(|| anyhow!("`terms.count` * `terms.amount` over maximum"))
      })
      .transpose()?
      .unwrap_or_default();

    ensure!(
      supply
        == premine
          .checked_add(mintable)
          .ok_or_else(|| anyhow!("`premine` + `terms.count` * `terms.amount` over maximum"))?,
      "`supply` not equal to `premine` + `terms.count` * `terms.amount`"
    );

    ensure!(supply > 0, "`supply` must be greater than zero");

    let bitcoin_client = wallet.bitcoin_client();

    let current_height = u32::try_from(bitcoin_client.get_block_count()?).unwrap();

    let reveal_height = current_height + 1 + u32::from(Runestone::COMMIT_INTERVAL);

    if let Some(terms) = etching.terms {
      if let Some((start, end)) = terms.offset.and_then(|range| range.start.zip(range.end)) {
        ensure!(
          end > start,
          "`terms.offset.end` must be greater than `terms.offset.start`"
        );
      }

      if let Some((start, end)) = terms.height.and_then(|range| range.start.zip(range.end)) {
        ensure!(
          end > start,
          "`terms.height.end` must be greater than `terms.height.start`"
        );
      }

      if let Some(end) = terms.height.and_then(|range| range.end) {
        ensure!(
          end > reveal_height.into(),
          "`terms.height.end` must be greater than the reveal transaction block height of {reveal_height}"
        );
      }

      if let Some(start) = terms.height.and_then(|range| range.start) {
        ensure!(
            start > reveal_height.into(),
            "`terms.height.start` must be greater than the reveal transaction block height of {reveal_height}"
          );
      }

      ensure!(terms.cap > 0, "`terms.cap` must be greater than zero");

      ensure!(
        terms.amount.to_integer(etching.divisibility)? > 0,
        "`terms.amount` must be greater than zero",
      );
    }

    let minimum = Rune::minimum_at_height(wallet.chain().into(), Height(reveal_height));

    ensure!(
      rune >= minimum,
      "rune is less than minimum for next block: {rune} < {minimum}",
    );

    Ok(())
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
parent: {parent}
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
            file: inscription_path,
            metadata: Some(Value::Mapping(metadata)),
            ..default()
          },
          batch::Entry {
            file: brc20_path,
            metaprotocol: Some("brc-20".to_string()),
            ..default()
          }
        ],
        parent: Some(parent),
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
