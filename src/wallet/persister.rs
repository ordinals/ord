use {super::*, bdk::chain::Merge};

pub(crate) struct TransactionPersister<'a>(pub(crate) &'a mut WriteTransaction);

impl WalletPersister for TransactionPersister<'_> {
  type Error = Error;

  fn initialize(persister: &mut Self) -> std::result::Result<ChangeSet, Self::Error> {
    let wtx = &persister.0;

    let changeset = match wtx
      .open_table(CHANGESET)?
      .get(())?
      .map(|result| result.value().to_string())
    {
      Some(result) => serde_json::from_str::<ChangeSet>(result.as_str())?,
      None => ChangeSet::default(),
    };

    Ok(changeset)
  }

  fn persist(persister: &mut Self, changeset: &ChangeSet) -> std::result::Result<(), Self::Error> {
    let mut current = Self::initialize(persister)?;

    current.merge(changeset.clone());

    let wtx = &persister.0;

    wtx
      .open_table(CHANGESET)?
      .insert((), serde_json::to_string(&current)?.as_str())?;

    Ok(())
  }
}

pub(crate) struct DatabasePersister(pub(crate) Arc<Database>);

impl WalletPersister for DatabasePersister {
  type Error = Error;

  fn initialize(persister: &mut Self) -> std::result::Result<ChangeSet, Self::Error> {
    TransactionPersister::initialize(&mut TransactionPersister(&mut persister.0.begin_write()?))
  }

  fn persist(persister: &mut Self, changeset: &ChangeSet) -> std::result::Result<(), Self::Error> {
    let mut wtx = persister.0.begin_write()?;

    TransactionPersister::persist(&mut TransactionPersister(&mut wtx), changeset)?;

    wtx.commit()?;

    Ok(())
  }
}
