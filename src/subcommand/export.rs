use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Export {
  #[clap(
    long,
    default_value = "inscriptions",
    help = "Write to directory <OUTPUT_DIR>"
  )]
  output_dir: PathBuf,
}

impl Export {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;
    self.run_with_index(&index)?;
    Ok(())
  }

  fn run_with_index(self, index: &Index) -> Result {
    fs::create_dir_all(&self.output_dir)?;
    let all_ids = index.get_inscriptions(None)?;
    for id in all_ids.values() {
      self.export_inscription_by_id(index, id.to_owned())?;
    }
    Ok(())
  }

  fn export_inscription_by_id(&self, index: &Index, id: InscriptionId) -> Result {
    let inscription = index
      .get_inscription_by_id(id)?
      .ok_or_else(|| anyhow!("inscription not found: {id}"))?;
    let entry = index
      .get_inscription_entry(id)?
      .ok_or_else(|| anyhow!("inscription entry not found: {id}"))?;
    let file = self.output_dir.join(format!("{}.txt", entry.number));
    let body = inscription
      .body()
      .ok_or_else(|| anyhow!("inscription body not found: {id}"))?;
    fs::write(file, body)?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::index::entry::InscriptionEntry;
  use crate::index::tests::Context;
  use crate::test::{inscription, InscriptionId};
  use test_bitcoincore_rpc::TransactionTemplate;

  fn write_test_inscription(context: &Context, contents: &str) -> Result<InscriptionEntry> {
    context.mine_blocks(1);
    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0)],
      witness: inscription("text/plain", contents).to_witness(),
      ..Default::default()
    });
    let inscription_id = InscriptionId::from(txid);

    context.mine_blocks(1);

    let entry = context
      .index
      .get_inscription_entry(inscription_id)?
      .ok_or_else(|| anyhow!("no entry for {inscription_id}"))?;
    Ok(entry)
  }

  #[test]
  fn writes_inscriptions_to_disk_by_number() -> Result {
    let context = Context::builder().build();
    let entry = write_test_inscription(&context, "foo")?;
    let export = Export {
      output_dir: context.tempdir.path().join("inscriptions"),
    };
    export.run_with_index(&context.index)?;
    assert_eq!(
      fs::read_to_string(
        context
          .tempdir
          .path()
          .join("inscriptions")
          .join(format!("{}.txt", entry.number))
      )?,
      "foo"
    );
    Ok(())
  }

  #[test]
  fn handles_inscriptions_without_bodies_gracefully() {}

  #[test]
  fn writes_other_media_types() {}

  #[test]
  fn aborts_gracefully_on_ctrl_c() {}

  #[test]
  fn avoids_rewriting_existing_files() {}

  #[test]
  fn shows_a_progress_bar() {}
}
