use {super::*, bitcoincore_rpc::RpcApi};

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.index()?;

  let client = options.rpc_client()?;

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

  for (start, size, output, rarity, name) in list(utxos) {
    println!("{start}\t{size}\t{output}\t{rarity}\t{name}");
  }

  Ok(())
}

fn list(utxos: Vec<(OutPoint, Vec<(u64, u64)>)>) -> Vec<(u64, u64, OutPoint, Rarity, String)> {
  utxos
    .into_iter()
    .flat_map(|(outpoint, ordinal_ranges)| {
      ordinal_ranges.into_iter().map(move |(start, end)| {
        let ordinal = Ordinal(start);
        let rarity = ordinal.rarity();
        let name = ordinal.name();
        let size = end - start;
        (start, size, outpoint, rarity, name)
      })
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn list_ranges() {
    let utxos = vec![
      (
        OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
          .unwrap(),
        vec![(50 * COIN_VALUE, 55 * COIN_VALUE)],
      ),
      (
        OutPoint::null(),
        vec![(10, 100), (1050000000000000, 1150000000000000)],
      ),
    ];
    assert_eq!(
      list(utxos),
      vec![
        (
          50 * COIN_VALUE,
          5 * COIN_VALUE,
          OutPoint::from_str("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5")
            .unwrap(),
          Rarity::Uncommon,
          "nvtcsezkbth".to_string()
        ),
        (
          10,
          90,
          OutPoint::null(),
          Rarity::Common,
          "nvtdijuwxlf".to_string()
        ),
        (
          1050000000000000,
          100000000000000,
          OutPoint::null(),
          Rarity::Epic,
          "gkjbdrhkfqf".to_string()
        )
      ]
    )
  }
}
