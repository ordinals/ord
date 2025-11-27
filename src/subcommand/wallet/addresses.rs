use super::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscriptions: Option<Vec<InscriptionId>>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let mut addresses: BTreeMap<Address<NetworkUnchecked>, Vec<Output>> = BTreeMap::new();

  for (output, txout) in wallet.utxos() {
    let address = wallet.chain().address_from_script(&txout.script_pubkey)?;

    let inscriptions = wallet.get_inscriptions_in_output(output)?;

    let output = Output {
      output: *output,
      amount: txout.value.to_sat(),
      inscriptions,
    };

    addresses
      .entry(address.as_unchecked().clone())
      .or_default()
      .push(output);
  }

  Ok(Some(Box::new(addresses)))
}
