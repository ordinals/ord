use super::*;

pub(crate) struct Database(());

impl Database {
  pub(crate) fn open(options: &Options) -> Result<Self> {
    todo!()
  }

  pub(crate) fn begin_write(&self) -> Result<WriteTransaction> {
    todo!()
  }

  pub(crate) fn print_info(&self) -> Result {
    todo!()
  }

  pub(crate) fn find(&self, ordinal: Ordinal) -> Result<Option<(u64, u64, SatPoint)>> {
    todo!()
  }

  pub(crate) fn list(&self, outpoint: &[u8]) -> Result<Vec<u8>> {
    todo!()
  }
}

pub(crate) struct WriteTransaction(());

impl WriteTransaction {
  pub(crate) fn abort(self) -> Result {
    todo!()
  }

  pub(crate) fn commit(self) -> Result {
    todo!()
  }

  pub(crate) fn height(&self) -> Result<u64> {
    todo!()
  }

  pub(crate) fn blockhash_at_height(&self, height: u64) -> Result<Option<&[u8]>> {
    todo!()
  }

  pub(crate) fn set_blockhash_at_height(&mut self, height: u64, blockhash: BlockHash) -> Result {
    todo!()
  }

  pub(crate) fn insert_outpoint(&mut self, outpoint: &[u8], ordinal_ranges: &[u8]) -> Result {
    todo!()
  }

  pub(crate) fn get_ordinal_ranges(&self, outpoint: &[u8]) -> Result<Option<&[u8]>> {
    todo!()
  }

  pub(crate) fn insert_satpoint(&mut self, key: &[u8], satpoint: &[u8]) -> Result {
    todo!()
  }
}
