use super::*;

pub(crate) struct TransactionOptions<'a> {
  pub(crate) slots: &'a [(usize, usize, usize)],
  pub(crate) output_count: usize,
  pub(crate) fee: u64,
  pub(crate) recipient: Option<Script>,
}
