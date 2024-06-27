use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum SnafuError {
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
