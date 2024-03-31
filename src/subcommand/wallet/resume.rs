use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResumeOutput {
  pub etchings: Vec<batch::Output>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let outputs: Result<Vec<batch::Output>, _> = wallet
    .pending()?
    .into_iter()
    .map(|(rune, entry)| {
      wallet.wait_for_maturation(&rune, entry.commit, entry.reveal, entry.output.clone())
    })
    .collect();

  outputs.map(|os| Some(Box::new(ResumeOutput { etchings: os }) as Box<dyn Output>))
}
