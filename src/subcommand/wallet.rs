use super::*;

mod identify;
mod list;
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
fn parse_tsv(tsv: &str) -> Result<Vec<(Ordinal, String)>> {
  let mut ordinals = Vec::new();
  for (i, line) in tsv.lines().enumerate() {
    if line.is_empty() || line.starts_with('#') {
      continue;
    }

    if let Some(value) = line.split('\t').next() {
      let ordinal = Ordinal::from_str(value).map_err(|err| {
        anyhow!(
          "failed to parse ordinal from string \"{value}\" on line {}: {err}",
          i + 1,
        )
      })?;

      ordinals.push((ordinal, value.to_string()));
    }
  }
  ordinals.sort();

  Ok(ordinals)
}

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  Identify(identify::Identify),
  List,
  Send(send::Send),
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Identify(identify) => identify.run(options),
      Self::List => list::run(options),
      Self::Send(send) => send.run(options),
    }
  }
}
