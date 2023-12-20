use {
  super::{
    ord::redb::{OrdDbReadWriter as OrdStateRW, OrdDbReader as OrdStateReader},
    StateRWriter, StateReader,
  },
  redb::{ReadTransaction, WriteTransaction},
};

/// StateReadOnly, based on `redb`, is an implementation of the StateRWriter trait.
pub struct StateReadOnly<'db, 'a> {
  ord: OrdStateReader<'db, 'a>,
}

impl<'db, 'a> StateReadOnly<'db, 'a> {
  #[allow(dead_code)]
  pub fn new(rtx: &'a ReadTransaction<'db>) -> Self {
    Self {
      ord: OrdStateReader::new(rtx),
    }
  }
}

impl<'db, 'a> StateReader for StateReadOnly<'db, 'a> {
  type OrdReader = OrdStateReader<'db, 'a>;

  fn ord(&self) -> &Self::OrdReader {
    &self.ord
  }
}

/// StateReadWrite, based on `redb`, is an implementation of the StateRWriter trait.
pub struct StateReadWrite<'db, 'a> {
  ord: OrdStateRW<'db, 'a>,
}

impl<'db, 'a> StateReadWrite<'db, 'a> {
  pub fn new(wtx: &'a WriteTransaction<'db>) -> Self {
    Self {
      ord: OrdStateRW::new(wtx),
    }
  }
}

impl<'db, 'a> StateRWriter for StateReadWrite<'db, 'a> {
  type OrdRWriter = OrdStateRW<'db, 'a>;

  fn ord(&self) -> &Self::OrdRWriter {
    &self.ord
  }
}
