use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct Output {
  pub inscriptions: Vec<Inscription>,
}

#[derive(Debug, Parser)]
pub(crate) struct Decode {
  transaction: PathBuf,
}

impl Decode {
  pub(crate) fn run(self) -> SubcommandResult {
    let mut file = File::open(self.transaction)?;

    let transaction = Transaction::consensus_decode(&mut file)?;

    let inscriptions = Inscription::from_transaction(&transaction);

    Ok(Box::new(Output {
      inscriptions: inscriptions
        .into_iter()
        .map(|inscription| inscription.inscription)
        .collect(),
    }))
  }
}
