use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct CompactOutput {
  pub inscriptions: Vec<CompactInscription>,
}

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct RawOutput {
  pub inscriptions: Vec<ParsedEnvelope>,
}

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct CompactInscription {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub body: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub content_encoding: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub content_type: Option<String>,
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  pub duplicate_field: bool,
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  pub incomplete_field: bool,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub metadata: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub metaprotocol: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub parent: Option<InscriptionId>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub pointer: Option<u64>,
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  pub unrecognized_even_field: bool,
}

impl TryFrom<Inscription> for CompactInscription {
  type Error = Error;

  fn try_from(inscription: Inscription) -> Result<Self> {
    Ok(Self {
      content_encoding: inscription
        .content_encoding()
        .map(|header_value| header_value.to_str().map(str::to_string))
        .transpose()?,
      content_type: inscription.content_type().map(str::to_string),
      metaprotocol: inscription.metaprotocol().map(str::to_string),
      parent: inscription.parent(),
      pointer: inscription.pointer(),
      body: inscription.body.map(hex::encode),
      duplicate_field: inscription.duplicate_field,
      incomplete_field: inscription.incomplete_field,
      metadata: inscription.metadata.map(hex::encode),
      unrecognized_even_field: inscription.unrecognized_even_field,
    })
  }
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
  #[arg(
    long,
    help = "Serialize inscriptions in a compact, human-readable format."
  )]
  compact: bool,
}

impl Decode {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let transaction = if let Some(txid) = self.txid {
      settings
        .bitcoin_rpc_client(None)?
        .get_raw_transaction(&txid, None)?
    } else if let Some(file) = self.file {
      Transaction::consensus_decode(&mut File::open(file)?)?
    } else {
      Transaction::consensus_decode(&mut io::stdin())?
    };

    let inscriptions = ParsedEnvelope::from_transaction(&transaction);

    if self.compact {
      Ok(Some(Box::new(CompactOutput {
        inscriptions: inscriptions
          .clone()
          .into_iter()
          .map(|inscription| inscription.payload.try_into())
          .collect::<Result<Vec<CompactInscription>>>()?,
      })))
    } else {
      Ok(Some(Box::new(RawOutput { inscriptions })))
    }
  }
}
