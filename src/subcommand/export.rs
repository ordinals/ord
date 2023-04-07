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
    let index = Arc::new(Index::open(&options)?);
    self.run_with_index(index)?;
    Ok(())
  }

  fn run_with_index(self, index: Arc<Index>) -> Result {
    fs::create_dir_all(&self.output_dir)?;
    let all_ids = index.get_inscriptions(None)?;
    for id in all_ids.values() {
      self.export_inscription_by_id(&index, id.to_owned())?;
      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break;
      }
    }
    Ok(())
  }

  fn export_inscription_by_id(&self, index: &Arc<Index>, id: InscriptionId) -> Result {
    let inscription = index
      .get_inscription_by_id(id)?
      .ok_or_else(|| anyhow!("inscription not found: {id}"))?;
    let entry = index
      .get_inscription_entry(id)?
      .ok_or_else(|| anyhow!("inscription entry not found: {id}"))?;
    let content_type = inscription
      .content_type()
      .ok_or_else(|| anyhow!("content_type missing for {id}"))?;
    let extension = Media::extension_for_content_type(content_type)
      .ok_or_else(|| anyhow!("unknown content_type: {content_type}"))?;
    let file = self
      .output_dir
      .join(format!("{}.{}", entry.number, extension));
    let body = inscription.body();
    match body {
      None => log::info!("inscription body not found: {id}"),
      Some(body) => fs::write(file, body)?,
    }
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

  impl Context {
    fn export_dir(&self) -> PathBuf {
      self.tempdir.path().join("inscriptions")
    }
    fn write_test_inscription(&self, inscription: Inscription) -> Result<InscriptionEntry> {
      self.mine_blocks(1);
      let txid = self.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0)],
        witness: inscription.to_witness(),
        ..Default::default()
      });
      let inscription_id = InscriptionId::from(txid);

      self.mine_blocks(1);

      let entry = self
        .index()
        .get_inscription_entry(inscription_id)?
        .ok_or_else(|| anyhow!("no entry for {inscription_id}"))?;
      Ok(entry)
    }
  }

  #[test]
  fn writes_inscriptions_to_disk_by_number() -> Result {
    let context = Context::builder().build();
    let entry = context.write_test_inscription(inscription("text/plain;charset=utf-8", "foo"))?;
    let export = Export {
      output_dir: context.export_dir(),
    };
    export.run_with_index(context.index())?;
    assert_eq!(
      fs::read_to_string(context.export_dir().join(format!("{}.txt", entry.number)))?,
      "foo"
    );
    Ok(())
  }

  #[test]
  fn writes_other_media_types() -> Result {
    let context = Context::builder().build();
    let entry = context.write_test_inscription(inscription("application/json", "{}"))?;
    let export = Export {
      output_dir: context.export_dir(),
    };
    export.run_with_index(context.index())?;
    assert_eq!(
      fs::read_to_string(context.export_dir().join(format!("{}.json", entry.number)))?,
      "{}"
    );
    Ok(())
  }

  #[test]
  fn skips_unsupported_media_types() -> Result {
    Ok(())
  }

  #[test]
  fn handles_inscriptions_without_bodies_gracefully() -> Result {
    let context = Context::builder().build();
    context.write_test_inscription(Inscription::new(
      Some("text/plain;charset=utf-8".into()),
      None,
    ))?;
    let export = Export {
      output_dir: context.export_dir(),
    };
    export.run_with_index(context.index())?;
    Ok(())
  }

  #[test]
  fn aborts_gracefully_on_ctrl_c() -> Result {
    let context = Context::builder().build();
    let n = 100;
    for _ in 0..n {
      context.write_test_inscription(inscription("text/plain;charset=utf-8", "foo"))?;
    }
    let thread = {
      let export = Export {
        output_dir: context.export_dir(),
      };
      let index = context.index();
      thread::spawn(|| -> Result {
        export.run_with_index(index)?;
        Ok(())
      })
    };
    SHUTTING_DOWN.store(true, atomic::Ordering::Relaxed);
    thread.join().unwrap()?;
    let written_files = fs::read_dir(context.export_dir())?.count();
    assert!(written_files > 0, "all {n} inscriptions written");
    assert!(written_files < n, "all {n} inscriptions written");
    Ok(())
  }

  #[test]
  fn avoids_rewriting_existing_files() -> Result {
    Ok(())
  }

  #[test]
  fn shows_a_progress_bar() -> Result {
    Ok(())
  }
}
