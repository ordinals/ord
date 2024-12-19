use super::*;

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct CompactOutput {
  pub inscriptions: Vec<CompactInscription>,
  pub runestone: Option<Artifact>,
}

#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct RawOutput {
  pub inscriptions: Vec<ParsedEnvelope>,
  pub runestone: Option<Artifact>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Eq, PartialEq, Deserialize, Debug)]
pub struct CompactInscription {
  pub body: Option<String>,
  pub content_encoding: Option<String>,
  pub content_type: Option<String>,
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  pub duplicate_field: bool,
  #[serde(default, skip_serializing_if = "std::ops::Not::not")]
  pub incomplete_field: bool,
  pub metadata: Option<String>,
  pub metaprotocol: Option<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub parents: Vec<InscriptionId>,
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
      parents: inscription.parents(),
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
      Transaction::consensus_decode(&mut io::BufReader::new(File::open(file)?))?
    } else {
      Transaction::consensus_decode(&mut io::BufReader::new(io::stdin()))?
    };

    let inscriptions = ParsedEnvelope::from_transaction(&transaction);

    let runestone = Runestone::decipher(&transaction);

    if self.compact {
      Ok(Some(Box::new(CompactOutput {
        inscriptions: inscriptions
          .clone()
          .into_iter()
          .map(|inscription| inscription.payload.try_into())
          .collect::<Result<Vec<CompactInscription>>>()?,
        runestone,
      })))
    } else {
      Ok(Some(Box::new(RawOutput {
        inscriptions,
        runestone,
      })))
    }
  }
}
