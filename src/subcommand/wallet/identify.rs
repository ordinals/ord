use super::*;

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.index()?;

  let client = Client::new(&options.rpc_url(), Auth::CookieFile(options.cookie_file()?))
    .context("Failed to connect to RPC URL")?;

  let utxos = client.list_unspent(None, None, None, None, None).unwrap();
  for utxo in utxos {
    let output = OutPoint::new(utxo.txid, utxo.vout);
    match index.list(output).unwrap() {
      Some(List::Unspent(ordinal_ranges)) => {
        for (offset, ordinal_range) in ordinal_ranges.iter().enumerate() {
          let ordinal = Ordinal(ordinal_range.0);
          let rarity = ordinal.rarity();
          match rarity {
            Rarity::Common => (),
            _ => println!("{ordinal}\t{output}\t{offset}\t{rarity}"),
          }
        }
      }
      Some(_) => (),
      None => (),
    }
  }

  Ok(())
}
