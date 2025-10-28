use {
  super::*,
  minicbor::{Decode, Encode},
};

#[derive(Debug, Decode, Encode, PartialEq)]
#[cbor(map)]
pub(crate) struct GalleryItem {
  #[n(0)]
  pub(crate) id: Option<InscriptionId>,
}
