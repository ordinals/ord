use super::*;

#[derive(Debug, PartialEq, Clone, DeserializeFromStr)]
pub(crate) enum Signer {
  Address(Address<NetworkUnchecked>),
  Inscription(InscriptionId),
  Output(OutPoint),
}

impl FromStr for Signer {
  type Err = SnafuError;

  fn from_str(input: &str) -> Result<Self, Self::Err> {
    if re::ADDRESS.is_match(input) {
      Ok(Signer::Address(
        input.parse().snafu_context(error::AddressParse { input })?,
      ))
    } else if re::OUTPOINT.is_match(input) {
      Ok(Signer::Output(
        input
          .parse()
          .snafu_context(error::OutPointParse { input })?,
      ))
    } else if re::INSCRIPTION_ID.is_match(input) {
      Ok(Signer::Inscription(
        input
          .parse()
          .snafu_context(error::InscriptionIdParse { input })?,
      ))
    } else {
      Err(SnafuError::SignerParse {
        input: input.to_string(),
      })
    }
  }
}
