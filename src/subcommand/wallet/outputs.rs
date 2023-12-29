use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(wallet: String, options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;

  index.update()?;

  let client = bitcoin_rpc_client_for_wallet_command(wallet, &options)?;

  let mut outputs = Vec::new();
  for (output, amount) in get_unspent_outputs(&client, &index)? {
    outputs.push(Output {
      output,
      amount: amount.to_sat(),
    });
  }

  Ok(Box::new(outputs))
}
