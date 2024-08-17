use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum SnafuError {
  #[snafu(display("Failed to parse address `{}`", input))]
  AddressParse {
    source: bitcoin::address::Error,
    input: String,
  },
  #[snafu(display("Failed to parse hash `{}`", input))]
  HashParse {
    source: bitcoin::hashes::hex::Error,
    input: String,
  },
  #[snafu(display("Failed to parse inscription ID `{}`", input))]
  InscriptionIdParse {
    source: inscriptions::inscription_id::ParseError,
    input: String,
  },
  #[snafu(display("Failed to parse integer `{}`", input))]
  IntegerParse {
    source: std::num::ParseIntError,
    input: String,
  },
  #[snafu(display("Failed to parse out point `{}`", input))]
  OutPointParse {
    source: bitcoin::transaction::ParseOutPointError,
    input: String,
  },
  #[snafu(display("Failed to parse rune `{}`", input))]
  RuneParse {
    source: ordinals::spaced_rune::Error,
    input: String,
  },
  #[snafu(display("Failed to parse sat `{}`", input))]
  SatParse {
    source: ordinals::sat::Error,
    input: String,
  },
  #[snafu(display("Failed to parse sat point `{}`", input))]
  SatPointParse {
    source: ordinals::sat_point::Error,
    input: String,
  },
  #[snafu(display("Unrecognized representation: `{}`", input))]
  UnrecognizedRepresentation { input: String },
  #[snafu(display("Unrecognized outgoing amount: `{}`", input))]
  AmountParse {
    source: bitcoin::amount::ParseAmountError,
    input: String,
  },
  #[snafu(display("Unrecognized outgoing: `{}`", input))]
  OutgoingParse { input: String },
  #[snafu(display("Failed to parse decimal: {}", source))]
  RuneAmountParse { source: error::Error, input: String },
  #[snafu(display("Invalid chain `{}`", chain))]
  InvalidChain { chain: String },
  #[snafu(display("Failed to convert script to address: {}", source))]
  AddressConversion { source: bitcoin::address::Error },
  #[snafu(display("{err}"))]
  Anyhow { err: anyhow::Error },
  #[snafu(display("environment variable `{variable}` not valid unicode: `{}`", value.to_string_lossy()))]
  EnvVarUnicode {
    backtrace: Backtrace,
    value: OsString,
    variable: String,
  },
  #[snafu(display("I/O error at `{}`", path.display()))]
  Io {
    backtrace: Backtrace,
    path: PathBuf,
    source: io::Error,
  },
}

impl From<Error> for SnafuError {
  fn from(err: Error) -> SnafuError {
    Self::Anyhow { err }
  }
}

/// We currently use `anyhow` for error handling but are migrating to typed
/// errors using `snafu`. This trait exists to provide access to
/// `snafu::ResultExt::{context, with_context}`, which are otherwise shadowed
/// by `anhow::Context::{context, with_context}`. Once the migration is
/// complete, this trait can be deleted, and `snafu::ResultExt` used directly.
pub(crate) trait ResultExt<T, E>: Sized {
  fn snafu_context<C, E2>(self, context: C) -> Result<T, E2>
  where
    C: snafu::IntoError<E2, Source = E>,
    E2: std::error::Error + snafu::ErrorCompat;

  #[allow(unused)]
  fn with_snafu_context<F, C, E2>(self, context: F) -> Result<T, E2>
  where
    F: FnOnce(&mut E) -> C,
    C: snafu::IntoError<E2, Source = E>,
    E2: std::error::Error + snafu::ErrorCompat;
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E> {
  fn snafu_context<C, E2>(self, context: C) -> Result<T, E2>
  where
    C: snafu::IntoError<E2, Source = E>,
    E2: std::error::Error + snafu::ErrorCompat,
  {
    use snafu::ResultExt;
    self.context(context)
  }

  fn with_snafu_context<F, C, E2>(self, context: F) -> Result<T, E2>
  where
    F: FnOnce(&mut E) -> C,
    C: snafu::IntoError<E2, Source = E>,
    E2: std::error::Error + snafu::ErrorCompat,
  {
    use snafu::ResultExt;
    self.with_context(context)
  }
}
