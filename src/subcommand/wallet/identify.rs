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

  let unspent = client.list_unspent(None, None, None, None, None)?;

  let mut utxos = Vec::new();
  for utxo in unspent {
    let output = OutPoint::new(utxo.txid, utxo.vout);
    match index.list(output)? {
      Some(List::Unspent(ordinal_ranges)) => {
        utxos.push((output, ordinal_ranges));
      }
      Some(List::Spent) => {
        bail!("Output {output} in wallet but is spent according to index")
      }
      None => bail!("Ordinals index has not seen {output}"),
    }
  }

  for (ordinal, output, offset, rarity) in identify(utxos) {
    println!("{ordinal}\t{output}\t{offset}\t{rarity}");
  }

  Ok(())
}

fn identify(utxos: Vec<(OutPoint, Vec<(u64, u64)>)>) -> Vec<(Ordinal, OutPoint, u64, Rarity)> {
  utxos
    .into_iter()
    .flat_map(|(outpoint, ordinal_ranges)| {
      let mut offset = 0;
      ordinal_ranges.into_iter().filter_map(move |(start, end)| {
        let ordinal = Ordinal(start);
        let rarity = ordinal.rarity();
        let start_offset = offset;
        offset += end - start;
        if rarity > Rarity::Common {
          Some((ordinal, outpoint, start_offset, rarity))
        } else {
          None
        }
      })
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn identify_no_rare_ordinals() {
    let utxos = vec![(
      OutPoint::null(),
      vec![(51 * COIN_VALUE, 100 * COIN_VALUE), (1234, 5678)],
    )];
    assert_eq!(identify(utxos), vec![])
  }

  #[test]
  fn identify_one_rare_ordinal() {
    let utxos = vec![(
      OutPoint::null(),
      vec![(10, 80), (50 * COIN_VALUE, 100 * COIN_VALUE)],
    )];
    assert_eq!(
      identify(utxos),
      vec![(
        Ordinal(50 * COIN_VALUE),
        OutPoint::null(),
        70,
        Rarity::Uncommon
      )]
    )
  }

  #[test]
  fn identify_two_rare_ordinals() {
    let utxos = vec![(
      OutPoint::null(),
      vec![(0, 100), (1050000000000000, 1150000000000000)],
    )];
    assert_eq!(
      identify(utxos),
      vec![
        (Ordinal(0), OutPoint::null(), 0, Rarity::Mythic),
        (
          Ordinal(1050000000000000),
          OutPoint::null(),
          100,
          Rarity::Epic
        )
      ]
    )
  }

  #[test]
  fn identify_rare_ordinals_in_different_outpoints() {
    let utxos = vec![
      (OutPoint::null(), vec![(50 * COIN_VALUE, 55 * COIN_VALUE)]),
      (
        OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
          .unwrap(),
        vec![(100 * COIN_VALUE, 111 * COIN_VALUE)],
      ),
    ];
    assert_eq!(
      identify(utxos),
      vec![
        (
          Ordinal(50 * COIN_VALUE),
          OutPoint::null(),
          0,
          Rarity::Uncommon
        ),
        (
          Ordinal(100 * COIN_VALUE),
          OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
            .unwrap(),
          0,
          Rarity::Uncommon
        )
      ]
    )
  }
}
