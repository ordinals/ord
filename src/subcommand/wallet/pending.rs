use {super::*, crate::wallet::entry::EtchingEntry};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PendingOutput {
  pub etchings: Vec<EtchingEntry>,
}
#[derive(Debug, Parser)]
pub(crate) struct Pending {}

impl Pending {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let etchings = wallet
      .pending_etchings()?
      .into_iter()
      .map(|(_, entry)| entry)
      .collect();

    Ok(Some(Box::new(PendingOutput { etchings }) as Box<dyn Output>))
  }
}
