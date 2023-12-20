pub(crate) mod ord;
pub(crate) mod protocol_manager;

pub use self::protocol_manager::ProtocolManager;
use {crate::Options, bitcoin::Network};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlockContext {
  pub network: Network,
  pub blockheight: u32,
  pub blocktime: u32,
}
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
  first_inscription_height: u32,
  enable_ord_receipts: bool,
  enable_index_bitmap: bool,
}

impl ProtocolConfig {
  pub(crate) fn new_with_options(options: &Options) -> Self {
    Self {
      first_inscription_height: options.first_inscription_height(),
      enable_ord_receipts: options.enable_save_ord_receipts,
      enable_index_bitmap: options.enable_index_bitmap,
    }
  }
}
