use super::*;

/// We currently use `anyhow` for error handling but are migrating to typed
/// errors using `snafu`. This trait exists to provide access to
/// `snafu::OptionExt::{context, with_context}`, which are otherwise shadowed
/// by `anyhow::Context::{context, with_context}`. Once the migration is
/// complete, this trait can be deleted, and `snafu::OptionExt` used directly.
pub trait OptionExt<T>: Sized {
  fn snafu_context<C, E>(self, context: C) -> Result<T, E>
  where
    C: snafu::IntoError<E, Source = snafu::NoneError>,
    E: std::error::Error + snafu::ErrorCompat;

  #[allow(unused)]
  fn with_snafu_context<F, C, E>(self, context: F) -> Result<T, E>
  where
    F: FnOnce() -> C,
    C: snafu::IntoError<E, Source = snafu::NoneError>,
    E: std::error::Error + snafu::ErrorCompat;
}

impl<T> OptionExt<T> for Option<T> {
  fn snafu_context<C, E>(self, context: C) -> Result<T, E>
  where
    C: snafu::IntoError<E, Source = snafu::NoneError>,
    E: std::error::Error + snafu::ErrorCompat,
  {
    snafu::OptionExt::context(self, context)
  }

  fn with_snafu_context<F, C, E>(self, context: F) -> Result<T, E>
  where
    F: FnOnce() -> C,
    C: snafu::IntoError<E, Source = snafu::NoneError>,
    E: std::error::Error + snafu::ErrorCompat,
  {
    snafu::OptionExt::with_context(self, context)
  }
}
