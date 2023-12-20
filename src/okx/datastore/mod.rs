pub mod ord;
mod redb;
mod script_key;

pub use self::{
  redb::{StateReadOnly, StateReadWrite},
  script_key::ScriptKey,
};

/// StateReader is a collection of multiple readonly storages.
///
/// There are multiple categories in the storage, and they can be obtained separately.
pub trait StateReader {
  type OrdReader: ord::DataStoreReadOnly;

  // Returns a reference to the readonly Ord store.
  fn ord(&self) -> &Self::OrdReader;
}

/// StateRWriter is a collection of multiple read-write storages.
///
/// There are multiple categories in the storage, and they can be obtained separately.
pub trait StateRWriter {
  type OrdRWriter: ord::DataStoreReadWrite;

  // Returns a reference to the read-write ord store.
  fn ord(&self) -> &Self::OrdRWriter;
}
