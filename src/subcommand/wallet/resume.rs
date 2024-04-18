use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResumeOutput {
  pub etchings: Vec<batch::Output>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let mut etchings = Vec::new();
  loop {
    if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
      break;
    }

    for (rune, entry) in wallet.pending_etchings()? {
      if wallet.is_mature(rune, &entry.commit)? {
        etchings.push(wallet.send_etching(rune, &entry)?);
      }
    }

    if wallet.pending_etchings()?.is_empty() {
      break;
    }

    if !wallet.integration_test() {
      thread::sleep(Duration::from_secs(5));
    }
  }

  Ok(Some(Box::new(ResumeOutput { etchings }) as Box<dyn Output>))
}
