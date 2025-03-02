use super::*;

pub(crate) fn create_database(wallet_name: &String, settings: &Settings) -> Result<Database> {
  let path = settings
    .data_dir()
    .join("wallets")
    .join(format!("{wallet_name}.redb"));

  if path.exists() {
    bail!(
      "wallet `{}` at `{}` already exists",
      wallet_name,
      path.display()
    );
  }

  if let Err(err) = fs::create_dir_all(path.parent().unwrap()) {
    bail!(
      "failed to create data dir `{}`: {err}",
      path.parent().unwrap().display()
    );
  }

  let database = Database::builder().create(&path)?;

  let mut tx = database.begin_write()?;
  tx.set_quick_repair(true);

  tx.open_table(CHANGESET)?;

  tx.open_table(XPRIV)?;

  tx.open_table(RUNE_TO_ETCHING)?;

  tx.open_table(STATISTICS)?
    .insert(&Statistic::Schema.key(), &SCHEMA_VERSION)?;

  tx.commit()?;

  Ok(database)
}

pub(crate) fn open_database(wallet_name: &String, settings: &Settings) -> Result<Database> {
  let path = settings
    .data_dir()
    .join("wallets")
    .join(format!("{wallet_name}.redb"));

  let db_path = path.clone().to_owned();
  let once = Once::new();
  let progress_bar = Mutex::new(None);
  let integration_test = settings.integration_test();

  let repair_callback = move |progress: &mut RepairSession| {
    once.call_once(|| {
      println!(
        "Wallet database file `{}` needs recovery. This can take some time.",
        db_path.display()
      )
    });

    if !(cfg!(test) || log_enabled!(log::Level::Info) || integration_test) {
      let mut guard = progress_bar.lock().unwrap();

      let progress_bar = guard.get_or_insert_with(|| {
        let progress_bar = ProgressBar::new(100);
        progress_bar.set_style(
          ProgressStyle::with_template("[repairing database] {wide_bar} {pos}/{len}").unwrap(),
        );
        progress_bar
      });

      #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
      progress_bar.set_position((progress.progress() * 100.0) as u64);
    }
  };

  let database = match Database::builder()
    .set_repair_callback(repair_callback)
    .open(&path)
  {
    Ok(database) => {
      {
        let schema_version = database
          .begin_read()?
          .open_table(STATISTICS)?
          .get(&Statistic::Schema.key())?
          .map(|x| x.value())
          .unwrap_or(0);

        match schema_version.cmp(&SCHEMA_VERSION) {
            cmp::Ordering::Less =>
              bail!(
                "wallet database at `{}` appears to have been built with an older, incompatible version of ord, consider deleting and rebuilding the index: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
                path.display()
              ),
            cmp::Ordering::Greater =>
              bail!(
                "wallet database at `{}` appears to have been built with a newer, incompatible version of ord, consider updating ord: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
                path.display()
              ),
            cmp::Ordering::Equal => {
            }
          }
      }

      database
    }
    Err(error) => bail!("failed to open wallet database: {error}"),
  };

  Ok(database)
}
