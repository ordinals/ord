use super::*;

// Custom Ordzaar Rune Response
// one of the reasons to create a custom response is to
// convert some of the bigint values into string
// and also to make the response consistent
// (prevent broken responses when bumping to the latest Ord version)

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneOutpoint {
  pub spaced_rune: SpacedRune,
  pub amount: String,
  pub divisibility: u8,
  pub symbol: Option<char>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneTerms {
  pub amount: Option<String>,
  pub cap: Option<String>,
  pub height: (Option<String>, Option<String>),
  pub offset: (Option<String>, Option<String>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneDetail {
  pub rune_id: RuneId,
  pub mintable: bool,
  pub rune: Rune,
  pub block: String,
  pub burned: String,
  pub divisibility: u8,
  pub etching: Txid,
  pub mints: String,
  pub number: String,
  pub premine: String,
  pub spaced_rune: SpacedRune,
  pub symbol: Option<char>,
  pub terms: Option<RuneTerms>,
  pub timestamp: String,
}

impl RuneOutpoint {
  pub fn from_spaced_rune_pile(spaced_rune_piled: (SpacedRune, Pile)) -> Self {
    Self {
      spaced_rune: spaced_rune_piled.0,
      amount: spaced_rune_piled.1.amount.to_string(),
      divisibility: spaced_rune_piled.1.divisibility,
      symbol: spaced_rune_piled.1.symbol,
    }
  }
}

impl RuneDetail {
  pub fn from_rune(rune_id: RuneId, entry: RuneEntry, mintable: bool) -> Self {
    let mut terms: Option<RuneTerms> = None;

    if let Some(terms_value) = entry.terms {
      terms = Some(RuneTerms {
        amount: match terms_value.amount {
          Some(v) => Some(v.to_string()),
          None => None,
        },
        cap: match terms_value.cap {
          Some(v) => Some(v.to_string()),
          None => None,
        },
        height: (
          match terms_value.height.0 {
            Some(v) => Some(v.to_string()),
            None => None,
          },
          match terms_value.height.1 {
            Some(v) => Some(v.to_string()),
            None => None,
          },
        ),
        offset: (
          match terms_value.offset.0 {
            Some(v) => Some(v.to_string()),
            None => None,
          },
          match terms_value.offset.1 {
            Some(v) => Some(v.to_string()),
            None => None,
          },
        ),
      })
    }

    Self {
      block: entry.block.to_string(),
      mintable,
      rune_id,
      rune: entry.spaced_rune.rune,
      spaced_rune: entry.spaced_rune,
      burned: entry.burned.to_string(),
      divisibility: entry.divisibility,
      etching: entry.etching,
      terms,
      mints: entry.mints.to_string(),
      premine: entry.premine.to_string(),
      number: entry.number.to_string(),
      symbol: entry.symbol,
      timestamp: entry.timestamp.to_string(),
    }
  }
}

pub fn str_coma_to_array(str_coma: &str) -> Vec<String> {
  str_coma.split(',').map(|s| s.trim().to_string()).collect()
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuneOutputBulkQuery {
  pub outpoints: String,
}
