use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Transactions {}

impl Transactions {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;
    index.update()?;

    let client = options.bitcoin_rpc_client()?;

    // let json: serde_json::Value = client.call("listtransactions", &[])?;
    // dbg!(&json);

    let txs = client
      .list_transactions(None, None, None, None)?
      .iter()
      .map(|tx| (tx.info.txid, tx.info.confirmations))
      .collect::<Vec<(Txid, i32)>>();

    for (txid, confirmations) in txs {
      println!("{txid}\t{confirmations}");
    }

    Ok(())
  }
}
