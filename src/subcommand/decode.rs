use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct Output {
  pub inscriptions: Vec<Inscription>,
}

#[derive(Debug, Parser)]
pub(crate) struct Decode {
  #[arg(
    long,
    conflicts_with = "file",
    help = "Fetch transaction with <TXID> from Bitcoin Core."
  )]
  txid: Option<Txid>,
  #[arg(long, conflicts_with = "txid", help = "Load transaction from <FILE>.")]
  file: Option<PathBuf>,
}

impl Decode {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let client = options.bitcoin_rpc_client()?;

    let transaction = if let Some(txid) = self.txid {
      client.get_raw_transaction(&txid, None)?
    } else if let Some(file) = self.file {
      Transaction::consensus_decode(&mut File::open(file)?)?
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
