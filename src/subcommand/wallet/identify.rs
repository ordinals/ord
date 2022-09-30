use {
  super::*,
  bitcoincore_rpc::{Auth, Client, RpcApi},
};

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.index()?;

  let cookie_file = options.cookie_file()?;
  let rpc_url = options.rpc_url();
  log::info!(
    "Connecting to Bitcoin Core RPC server at {rpc_url} using credentials from `{}`",
    cookie_file.display()
  );

  let client = Client::new(&rpc_url, Auth::CookieFile(cookie_file))
    .context("Failed to connect to Bitcoin Core RPC at {rpc_url}")?;

  let utxos = client.list_unspent(None, None, None, None, None)?;

  for utxo in utxos {
    let output = OutPoint::new(utxo.txid, utxo.vout);
    match index.list(output)? {
      Some(List::Unspent(ordinal_ranges)) => {
        let mut offset = 0;
        for range in &ordinal_ranges {
          let ordinal = Ordinal(range.0);
          let rarity = ordinal.rarity();
          if rarity > Rarity::Common {
            println!("{ordinal}\t{output}\t{offset}\t{rarity}");
          }
          offset += range.1 - range.0
        }
      }
      Some(List::Spent(_)) => {
        bail!("Output {output} in wallet but is spent according to index")
      }
      None => bail!("Ordinals index has not seen {output}"),
    }
  }

  Ok(())
}
