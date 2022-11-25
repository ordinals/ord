use {super::*, transaction_builder::TransactionBuilder};

mod identify;
mod inscribe;
mod send;
mod transaction_builder;

fn list_unspent(options: &Options, index: &Index) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
  let client = options.bitcoin_rpc_client()?;

  client
    .list_unspent(None, None, None, None, None)?
    .iter()
    .map(|utxo| {
      let outpoint = OutPoint::new(utxo.txid, utxo.vout);
      match index.list(outpoint)? {
        Some(List::Unspent(ordinal_ranges)) => Ok((outpoint, ordinal_ranges)),
        Some(List::Spent) => bail!("output {outpoint} in wallet but is spent according to index"),
        None => bail!("ordinals index has not seen {outpoint}"),
      }
    })
    .collect()
}

fn get_change_addresses(options: &Options, n: usize) -> Result<Vec<Address>> {
  let client = options.bitcoin_rpc_client()?;

  let mut addresses = Vec::new();
  for _ in 0..n {
    addresses.push(
      client
        .call("getrawchangeaddress", &[])
        .context("could not get change addresses from wallet")?,
    );
  }

  Ok(addresses)
}

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  Identify(identify::Identify),
  Inscribe(inscribe::Inscribe),
  Send(send::Send),
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Identify(identify) => identify.run(options),
      Self::Inscribe(inscribe) => inscribe.run(options),
      Self::Send(send) => send.run(options),
    }
  }
}
