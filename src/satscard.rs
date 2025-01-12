use super::*;

#[derive(Deserialize)]
pub(crate) struct Query {
  pub(crate) u: String, // S
  pub(crate) o: u64,    // 0
  pub(crate) r: String, // a5x2tplf, fingerprint?
  pub(crate) n: String, // 7664168a4ef7b8e8, nonce?
  pub(crate) s: String, // 42b209c86ab90be6418d36b0accc3a53c11901861b55be95b763799842d403dc17cd1b74695a7ffe2d78965535d6fe7f6aafc77f6143912a163cb65862e8fb53, address?
}
