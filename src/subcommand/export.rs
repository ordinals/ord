use {
  super::*,
  crate::index::entry::InscriptionEntry,
  indicatif::{ProgressIterator, ProgressStyle},
  log::log_enabled,
};

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
  pub(crate) fn run(&self, options: Options) -> Result {
    let index = Arc::new(Index::open(&options)?);
    index.update()?;
    self.run_with_index(index)?;
    Ok(())
  }

  fn run_with_index(&self, index: Arc<Index>) -> Result {
    self.prepare_directory_tree()?;
    let written_numbers = self.get_written_numbers()?;
    let ids = index.get_inscriptions(None)?;
    let index = index.as_ref();
    for id in Export::add_progress_bar(ids.into_values()) {
      Export::retry(|| {
        let entry = index
          .get_inscription_entry(id)?
          .ok_or_else(|| anyhow!("inscription entry not found: {id}"))?;
        if !written_numbers.contains(&entry.number) {
          self.export_inscription_by_id(index, id, entry)?;
        }
        Ok(())
      })?;
      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break;
      }
    }
    Ok(())
  }

  fn prepare_directory_tree(&self) -> Result {
    fs::create_dir_all(self.numbers_dir())?;
    Ok(())
  }

  fn numbers_dir(&self) -> PathBuf {
    self.output_dir.join("numbers")
  }

  fn get_written_numbers(&self) -> Result<HashSet<u64>> {
    Ok(
      fs::read_dir(self.numbers_dir())?
        .filter_map(|dir_entry| -> Option<u64> {
          u64::from_str(dir_entry.ok()?.path().file_stem()?.to_str()?).ok()
        })
        .collect(),
    )
  }

  fn export_inscription_by_id(
    &self,
    index: &Index,
    id: InscriptionId,
    entry: InscriptionEntry,
  ) -> Result {
    let inscription = index
      .get_inscription_by_id(id)?
      .ok_or_else(|| anyhow!("inscription not found: {id}"))?;
    let number = entry.number;
    match inscription.body() {
      None => log::info!("inscription {number} has no body"),
      Some(body) => {
        let file = self.file_name(&inscription, number);
        fs::write(file, body)?;
      }
    }
    Ok(())
  }

  fn file_name(&self, inscription: &Inscription, number: u64) -> PathBuf {
    let mut file = self.numbers_dir().join(number.to_string());
    let extension = Export::file_extension(inscription, number);
    if let Some(extension) = extension {
      file.set_extension(extension);
    }
    file
  }

  fn file_extension(inscription: &Inscription, number: u64) -> Option<&'static str> {
    match inscription.content_type() {
      None => {
        log::info!("content_type missing, writing inscription {number} without file extension");
        None
      }
      Some(content_type) => match Media::extension_for_content_type(content_type) {
        Err(message) => {
          log::info!("{message}, writing inscription {number} without file extension");
          None
        }
        Ok(extension) => Some(extension),
      },
    }
  }

  fn add_progress_bar<Iter, T>(iterator: Iter) -> Box<(dyn Iterator<Item = T>)>
  where
    Iter: Iterator<Item = T> + ExactSizeIterator + 'static,
  {
    if cfg!(test) || log_enabled!(log::Level::Info) || integration_test() {
      Box::new(iterator)
    } else {
      Box::new(
        iterator.into_iter().progress_with_style(
          ProgressStyle::with_template("[exporting inscriptions] {{wide_bar}} {{pos}}/{{len}}")
            .unwrap(),
        ),
      )
    }
  }

  fn retry<F, T>(mut f: F) -> Result<T>
  where
    F: FnMut() -> Result<T>,
  {
    let mut tries = 5;
    loop {
      match f() {
        Err(e) => {
          if tries > 0 {
            tries -= 1;
          } else {
            return Err(e);
          }
        }
        Ok(t) => return Ok(t),
      }
    }
  }
}

#[cfg(test)]
mod test {
  use {
    super::*,
    crate::{index::tests::Context, test::inscription},
    pretty_assertions::assert_eq,
  };

  impl Context {
    fn export_dir(&self) -> PathBuf {
      self.tempdir.path().join("inscriptions")
    }

    fn run_export(&self) -> Result {
      let export = Export {
        output_dir: self.export_dir(),
      };
      export.run_with_index(self.index())?;
      Ok(())
    }
  }

  #[test]
  fn write_inscriptions_to_disk_by_number() -> Result {
    let context = Context::builder().build();
    let entry =
      context.write_test_inscription(1, inscription("text/plain;charset=utf-8", "foo"))?;
    context.run_export()?;
    assert_eq!(
      fs::read_to_string(
        context
          .export_dir()
          .join(format!("numbers/{}.txt", entry.number))
      )?,
      "foo"
    );
    Ok(())
  }

  #[test]
  fn write_multiple_inscriptions() -> Result {
    let context = Context::builder().build();
    let a = context.write_test_inscription(1, inscription("text/plain;charset=utf-8", "foo"))?;
    let b = context.write_test_inscription(2, inscription("text/plain;charset=utf-8", "bar"))?;
    context.run_export()?;
    assert_eq!(
      fs::read_to_string(
        context
          .export_dir()
          .join(format!("numbers/{}.txt", a.number))
      )?,
      "foo"
    );
    assert_eq!(
      fs::read_to_string(
        context
          .export_dir()
          .join(format!("numbers/{}.txt", b.number))
      )?,
      "bar"
    );
    Ok(())
  }

  #[test]
  fn other_content_types() -> Result {
    let context = Context::builder().build();
    let entry = context.write_test_inscription(1, inscription("application/json", "{}"))?;
    context.run_export()?;
    assert_eq!(
      fs::read_to_string(
        context
          .export_dir()
          .join("numbers")
          .join(format!("{}.json", entry.number))
      )?,
      "{}"
    );
    Ok(())
  }

  #[test]
  fn write_unsupported_content_types_without_extensions() -> Result {
    let context = Context::builder().build();
    let entry = context.write_test_inscription(1, inscription("something unsupported", "foo"))?;
    context.run_export()?;
    assert_eq!(
      fs::read_to_string(
        context
          .export_dir()
          .join("numbers")
          .join(format!("{}", entry.number))
      )?,
      "foo"
    );
    Ok(())
  }

  #[test]
  fn content_types_without_charset_parameter() -> Result {
    let context = Context::builder().build();
    let text_entry = context.write_test_inscription(1, inscription("text/plain", "foo"))?;
    let html_entry = context.write_test_inscription(2, inscription("text/html", "<foo/>"))?;
    context.run_export()?;
    assert_eq!(
      fs::read_to_string(
        context
          .export_dir()
          .join("numbers")
          .join(format!("{}.txt", text_entry.number))
      )?,
      "foo"
    );
    assert_eq!(
      fs::read_to_string(
        context
          .export_dir()
          .join("numbers")
          .join(format!("{}.html", html_entry.number))
      )?,
      "<foo/>"
    );
    Ok(())
  }

  #[test]
  fn write_inscriptions_without_content_types_without_file_extension() -> Result {
    let context = Context::builder().build();
    let entry = context.write_test_inscription(1, Inscription::new(None, Some("foo".into())))?;
    context.run_export()?;
    assert_eq!(
      fs::read_to_string(
        context
          .export_dir()
          .join("numbers")
          .join(entry.number.to_string())
      )?,
      "foo"
    );
    Ok(())
  }

  #[test]
  fn handle_inscriptions_without_bodies_gracefully() -> Result {
    let context = Context::builder().build();
    context.write_test_inscription(
      1,
      Inscription::new(Some("text/plain;charset=utf-8".into()), None),
    )?;
    context.run_export()?;
    Ok(())
  }

  #[test]
  fn abort_gracefully_on_ctrl_c() -> Result {
    let context = Context::builder().build();
    let n = 100;
    for i in 0..n {
      context.write_test_inscription(i + 1, inscription("text/plain;charset=utf-8", "foo"))?;
    }
    let thread = {
      let export = Export {
        output_dir: context.export_dir(),
      };
      let index = context.index();
      thread::spawn(move || -> Result {
        export.run_with_index(index)?;
        Ok(())
      })
    };
    SHUTTING_DOWN.store(true, atomic::Ordering::Relaxed);
    thread.join().unwrap()?;
    let written_files = fs::read_dir(context.export_dir())?.count();
    assert!(written_files > 0, "no inscriptions written");
    assert!(written_files < n, "all {n} inscriptions written");
    Ok(())
  }

  #[test]
  fn avoid_rewriting_existing_files() -> Result {
    let context = Context::builder().build();
    let entry = context.write_test_inscription(1, inscription("text/plain", "foo"))?;
    context.run_export()?;
    let file = context
      .export_dir()
      .join("numbers")
      .join(format!("{}.txt", entry.number));
    fs::write(&file, "bar")?;
    context.run_export()?;
    assert_eq!(fs::read_to_string(file)?, "bar");
    Ok(())
  }

  #[test]
  fn retry_succeeds_for_single_failures() {
    fn f(first: &mut bool) -> Result {
      if *first {
        *first = false;
        Err(anyhow!("fails first"))
      } else {
        Ok(())
      }
    }
    let mut first = true;
    assert_eq!(
      Export::retry(|| f(&mut first)).map_err(|e| e.to_string()),
      Ok(())
    );
  }

  #[test]
  fn retry_fails_on_persisting_failures() {
    fn f() -> Result {
      Err(anyhow!("fails always"))
    }
    assert_eq!(
      Export::retry(f).map_err(|e| e.to_string()),
      Err("fails always".to_string())
    );
  }
}
