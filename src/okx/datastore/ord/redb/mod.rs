pub mod read_only;
pub mod read_write;

pub use self::{
  read_only::OrdDbReader,
  read_write::{try_init_tables, OrdDbReadWriter},
};
use {super::CollectionKind, redb::TableDefinition};

const ORD_TX_TO_OPERATIONS: TableDefinition<&str, &[u8]> =
  TableDefinition::new("ORD_TX_TO_OPERATIONS");
const COLLECTIONS_KEY_TO_INSCRIPTION_ID: TableDefinition<&str, &[u8; 36]> =
  TableDefinition::new("COLLECTIONS_KEY_TO_INSCRIPTION_ID");
const COLLECTIONS_INSCRIPTION_ID_TO_KINDS: TableDefinition<&[u8; 36], &[u8]> =
  TableDefinition::new("COLLECTIONS_INSCRIPTION_ID_TO_KINDS");
