use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Flaw {
  EdictOutput,
  EdictRuneId,
  InvalidScript,
  Opcode,
  SupplyOverflow,
  TrailingIntegers,
  TruncatedField,
  UnrecognizedEvenTag,
  UnrecognizedFlag,
  Varint,
}

impl Display for Flaw {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::EdictOutput => write!(f, "edict output greater than transaction output count"),
      Self::EdictRuneId => write!(f, "invalid rune ID in edict"),
      Self::InvalidScript => write!(f, "invalid script in OP_RETURN"),
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
