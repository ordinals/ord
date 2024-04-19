use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResumeOutput {
  pub etchings: Vec<batch::Output>,
}
#[derive(Debug, Parser)]
pub(crate) struct Resume {
  #[arg(long, help = "Don't broadcast transactions.")]
  pub(crate) dry_run: bool,
}

impl Resume {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let mut etchings = Vec::new();
    loop {
      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break;
      }

      for (rune, entry) in wallet.pending_etchings()? {
        if self.dry_run {
          etchings.push(batch::Output {
            reveal_broadcast: false,
            ..entry.output.clone()
          });
          continue;
        };

        if wallet.is_mature(rune, &entry.commit)? {
          etchings.push(wallet.send_etching(rune, &entry)?);
        }
      }

      if wallet.pending_etchings()?.is_empty() {
        break;
      }

      if self.dry_run {
        break;
      }

      if !wallet.integration_test() {
        thread::sleep(Duration::from_secs(5));
      }
    }

    Ok(Some(Box::new(ResumeOutput { etchings }) as Box<dyn Output>))
  }
}
