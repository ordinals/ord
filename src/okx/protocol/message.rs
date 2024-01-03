use crate::okx::datastore::brc20 as brc20_store;
use crate::okx::protocol::brc20 as brc20_proto;

#[allow(clippy::upper_case_acronyms)]
pub enum Message {
  BRC20(brc20_proto::Message),
}

#[allow(clippy::upper_case_acronyms)]
#[allow(unused)]
pub enum Receipt {
  BRC20(brc20_store::Receipt),
}
