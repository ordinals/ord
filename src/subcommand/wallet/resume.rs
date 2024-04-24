use crate::wallet::MaturityFailureStatus;
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

      for (index, (rune, entry)) in wallet.pending_etchings()?.iter().enumerate() {
        if self.dry_run {
          etchings.push(batch::Output {
            reveal_broadcast: false,
            ..entry.output.clone()
          });
          continue;
        };
        let rune_maturity = wallet.check_rune_maturity(rune.clone(), &entry.commit)?;
        if rune_maturity.matured {
          etchings.push(wallet.send_etching(rune.clone(), &entry)?);
        }

        if let Some(maturity_failure_status) = rune_maturity.maturity_failure_status {
          match maturity_failure_status {
            MaturityFailureStatus::CommitSpent(tx_id) => {
              // shouldn't to bail out here it can proceed with other pending etchings.
              eprintln!("Commitment {} Spent", tx_id);
              etchings.remove(index);
            }
            _ => continue
          }
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
