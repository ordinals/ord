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

  let mut utxos = Vec::new();
  for utxo in client.list_unspent(None, None, None, None, None)? {
    let outpoint = OutPoint::new(utxo.txid, utxo.vout);
    match index.list(outpoint)? {
      Some(List::Unspent(ordinal_ranges)) => utxos.push((outpoint, ordinal_ranges)),
      Some(List::Spent) => bail!("Output {outpoint} in wallet but is spent according to index"),
      None => bail!("Ordinals index has not seen {outpoint}"),
    }
  }

  Ok(utxos)
}

#[cfg(test)]
mod tests {}
