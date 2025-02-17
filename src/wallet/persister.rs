use {
  super::*,
  bdk::{ChangeSet, WalletPersister},
};

pub(crate) struct Persister(pub(crate) Arc<Database>);

impl WalletPersister for Persister {
  type Error = Error;

  fn initialize(persister: &mut Self) -> std::result::Result<bdk_wallet::ChangeSet, Self::Error> {
    Ok(ChangeSet::default())
  }

  fn persist(
    persister: &mut Self,
    changeset: &bdk_wallet::ChangeSet,
  ) -> std::result::Result<(), Self::Error> {
    let wtx = persister.0.begin_write()?;

    wtx
      .open_table(CHANGESET)?
      .insert((), serde_json::to_string(changeset)?.as_str())?;

    wtx.commit()?;

    Ok(())
  }
}
