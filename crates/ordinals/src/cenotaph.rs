use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cenotaph {
  EdictOutput,
  EdictRuneId,
  Opcode,
  SupplyOverflow,
  TrailingIntegers,
  TruncatedField,
  UnrecognizedEvenTag,
  UnrecognizedFlag,
  Varint,
}

impl Cenotaph {
  pub const ALL: [Self; 9] = [
    Self::EdictOutput,
    Self::EdictRuneId,
    Self::Opcode,
    Self::SupplyOverflow,
    Self::TrailingIntegers,
    Self::TruncatedField,
    Self::UnrecognizedEvenTag,
    Self::UnrecognizedFlag,
    Self::Varint,
  ];

  pub fn flag(self) -> u32 {
    1 << (self as u32)
  }
}

impl Display for Cenotaph {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::EdictOutput => write!(f, "edict output greater than transaction output count"),
      Self::EdictRuneId => write!(f, "invalid rune ID in edict"),
      Self::Opcode => write!(f, "non-pushdata opcode in OP_RETURN"),
      Self::SupplyOverflow => write!(f, "supply overflows u128"),
      Self::TrailingIntegers => write!(f, "trailing integers in body"),
      Self::TruncatedField => write!(f, "field with missing value"),
      Self::UnrecognizedEvenTag => write!(f, "unrecognized even tag"),
      Self::UnrecognizedFlag => write!(f, "unrecognized field"),
      Self::Varint => write!(f, "invalid varint"),
    }
  }
}

impl From<Cenotaph> for Runestone {
  fn from(cenotaph: Cenotaph) -> Self {
    Self {
      cenotaph: cenotaph.flag(),
      ..default()
    }
  }
}

impl From<Cenotaph> for u32 {
  fn from(cenotaph: Cenotaph) -> Self {
    cenotaph.flag()
  }
}
