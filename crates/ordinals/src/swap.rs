use super::*;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq)]
pub struct Swap {
  // specifies input token, defaults to UNCOMMON•GOODS
  pub input: Option<RuneId>,
  // specifies output token, defaults to UNCOMMON•GOODS
  pub output: Option<RuneId>,
  // min/max amount of input tokens
  pub input_amount: Option<u128>,
  // min/max amount of output tokens
  pub output_amount: Option<u128>,
  // if false, this is an exact-output order
  // if true, this is an exact-input order
  pub is_exact_input: bool,
}
