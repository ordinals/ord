use {super::*, crate::wallet::Maturity};

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

      for (i, (rune, entry)) in wallet.pending_etchings()?.iter().enumerate() {
        if self.dry_run {
          etchings.push(batch::Output {
            reveal_broadcast: false,
            ..entry.output.clone()
          });
          continue;
        };

        match wallet.check_maturity(*rune, &entry.commit)? {
          Maturity::Mature => etchings.push(wallet.send_etching(*rune, entry)?),
          Maturity::CommitSpent(txid) => {
            eprintln!("Commitment for rune etching {rune} spent in {txid}");
            etchings.remove(i);
          }
          Maturity::CommitNotFound => {
            eprintln!("Commit not found for rune etching {rune}");
            etchings.remove(i);
          }
          Maturity::BelowMinimumHeight(_) => {}
          Maturity::ConfirmationsPending(_) => {}
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
