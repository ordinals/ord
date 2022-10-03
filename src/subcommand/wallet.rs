use {super::*, bitcoincore_rpc::RpcApi};

mod identify;
mod list;

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  Identify,
  List,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    match self {
      Self::Identify => identify::run(options),
      Self::List => list::run(options),
    }
  }
}

fn list_unspent(options: Options) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
  let index = Index::open(&options)?;
  index.index()?;

  let client = options.bitcoin_rpc_client()?;

  client
    .list_unspent(None, None, None, None, None)?
    .iter()
    .map(|utxo| {
      let output = OutPoint::new(utxo.txid, utxo.vout);
      Ok((output, index.list_unspent(output)?))
    })
    .collect()
}

#[cfg(test)]
mod tests {}
