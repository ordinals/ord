use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResumeOutput {
  pub etchings: Vec<batch::Output>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let outputs: Result<Vec<batch::Output>> = wallet
    .pending_etchings()?
    .into_iter()
    .map(|(rune, entry)| {
      wallet.wait_for_maturation(&rune, entry.commit, entry.reveal, entry.output)
    })
    .collect();

  outputs.map(|etchings| Some(Box::new(ResumeOutput { etchings }) as Box<dyn Output>))
}
