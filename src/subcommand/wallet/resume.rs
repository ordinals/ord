use {super::*, crate::wallet::Maturity};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ResumeOutput {
  pub etchings: Vec<batch::Output>,
}
#[derive(Debug, Parser)]
pub(crate) struct Resume {
  #[arg(long, help = "Don't broadcast transactions.")]
  pub(crate) dry_run: bool,

  #[arg(long, help = "Rune commitment to resume.")]
  pub(crate) commitment: Option<String>,
}

impl Resume {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let mut etchings = Vec::new();
    loop {
      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break;
      }

      let mut pending_etchings = Vec::new();
      let commitment = self.commitment.clone();

      if let Some(commitment) = commitment {
        let pending_etching = wallet
          .pending_etchings()?
          .into_iter()
          .find(|(_, entry)| entry.commit.txid().to_string() == commitment);

        ensure!(
          pending_etching.is_some(),
          "commitment `{commitment}` does not correspond to any pending rune etching."
        );

        pending_etchings.push(pending_etching.unwrap());
      } else {
        pending_etchings.extend(wallet.pending_etchings()?.into_iter());
      }

      for (rune, entry) in pending_etchings {
        if self.dry_run {
          etchings.push(batch::Output {
            reveal_broadcast: false,
            ..entry.output.clone()
          });
          continue;
        };

        match wallet.check_maturity(rune, &entry.commit)? {
          Maturity::Mature => etchings.push(wallet.send_etching(rune, &entry)?),
          Maturity::CommitSpent(txid) => {
            eprintln!("Commitment for rune etching {rune} spent in {txid}");
            wallet.clear_etching(rune)?;
          }
          Maturity::CommitNotFound => {}
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
