use {super::*, bitcoincore_rpc::RpcApi};

mod identify;
mod list;

fn list_unspent(options: Options) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
  let index = Index::open(&options)?;
  index.index()?;

  let client = options.bitcoin_rpc_client()?;

  client
    .list_unspent(None, None, None, None, None)?
    .iter()
    .map(|utxo| {
      let outpoint = OutPoint::new(utxo.txid, utxo.vout);
      match index.list(outpoint)? {
        Some(List::Unspent(ordinal_ranges)) => Ok((outpoint, ordinal_ranges)),
        Some(List::Spent) => bail!("Output {outpoint} in wallet but is spent according to index"),
        None => bail!("Ordinals index has not seen {outpoint}"),
      }
    })
    .collect()
}

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

#[cfg(test)]
mod tests {}
