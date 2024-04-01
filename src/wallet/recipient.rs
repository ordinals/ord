use rustls_acme::acme::AuthStatus::Invalid;
use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum RecipientError {
  InvalidRecipient
}

impl Display for RecipientError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      RecipientError::InvalidRecipient => {
        write!(f, "Invalid recipient")
      }
    }
  }
}

impl std::error::Error for RecipientError {}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Recipient {
  Address(Address),
  ScriptBuf(ScriptBuf),
}

trait ScriptPubkey {
  fn script_pubkey(&self) -> ScriptBuf;
}

impl ScriptPubkey for Address {
  fn script_pubkey(&self) -> ScriptBuf {
    self.script_pubkey()
  }
}

impl ScriptPubkey for ScriptBuf {
  fn script_pubkey(&self) -> ScriptBuf {
    self.clone()
  }
}

impl Recipient {
  pub fn to_address(&self) -> Result<Address> {
    match self {
      Recipient::Address(address) => Ok(address.clone()),
      _ => Err(anyhow!(RecipientError::InvalidRecipient)),
    }
  }
  pub fn script_pubkey(&self) -> ScriptBuf {
    match self {
      Recipient::Address(address) => address.script_pubkey(),
      Recipient::ScriptBuf(script_buf) => script_buf.script_pubkey(),
    }
  }
}