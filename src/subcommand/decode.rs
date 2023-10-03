use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct Output {
  pub inscriptions: Vec<Inscription>,
}

#[derive(Debug, Parser)]
pub(crate) struct Decode {
  transaction: Option<PathBuf>,
}

impl Decode {
  pub(crate) fn run(self) -> SubcommandResult {
    let transaction = if let Some(path) = self.transaction {
      Transaction::consensus_decode(&mut File::open(path)?)?
    } else {
      Transaction::consensus_decode(&mut io::stdin())?
    };

    let inscriptions = ParsedEnvelope::from_transaction(&transaction);

    Ok(Box::new(Output {
      inscriptions: inscriptions
        .into_iter()
        .map(|inscription| inscription.payload)
        .collect(),
    }))
  }
}
