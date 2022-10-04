use super::*;

// todo:
// - add table headers
// - center everything
// - fix rarity color
// - add decimal
//
// time:
// - block height
// - date

#[derive(Boilerplate)]
pub(crate) struct RareTxt(pub(crate) Vec<(Ordinal, SatPoint)>);
