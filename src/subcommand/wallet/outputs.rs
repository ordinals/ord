use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(wallet_name: String, options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;

  index.update()?;

  let wallet_client = bitcoin_rpc_client_for_wallet_command(wallet_name, &options)?;

  let mut outputs = Vec::new();
  for (output, amount) in wallet::get_unspent_outputs(&wallet_client, &index)? {
    outputs.push(Output {
      output,
      amount: amount.to_sat(),
    });
  }

  Ok(Box::new(outputs))
}
