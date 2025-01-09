use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum Role {
  Seller,
  Buyer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub balance_change: i64,
  pub buyer_address: Address<NetworkUnchecked>,
  pub fee: Amount,
  pub fee_rate: FeeRate,
  pub inscription: InscriptionId,
  pub role: Role,
  pub seller_address: Address<NetworkUnchecked>,
}

#[derive(Debug, Parser)]
pub(crate) struct View {
  #[arg(long, help = "View <PSBT>")]
  psbt: String,
}

impl View {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let psbt = Psbt::deserialize(&base64_decode(&self.psbt)?)?;

    let mut outgoing = BTreeSet::new();

    for input in &psbt.unsigned_tx.input {
      if wallet.utxos().contains_key(&input.previous_output) {
        outgoing.insert(input.previous_output);
      }
    }

    todo!()
  }
}
