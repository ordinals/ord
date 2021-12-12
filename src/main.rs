use {
  bitcoin::blockdata::transaction::OutPoint,
  bitcoincore_rpc::{Auth, Client, RpcApi},
  std::collections::BTreeMap,
};

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn main() -> Result<()> {
  let home = dirs::home_dir().ok_or("Failed to retrieve home dir.")?;

  let cookiefile = home.join("Library/Application Support/Bitcoin/.cookie");

  if !cookiefile.is_file() {
    return Err(
      format!(
        "Bitcoin RPC cookiefile does not exist. Path: {}",
        cookiefile.display()
      )
      .into(),
    );
  }

  let client = Client::new("http://localhost:8332", Auth::CookieFile(cookiefile))?;

  let tip_height = client
    .get_block_header_info(&client.get_best_block_hash()?)?
    .height as u64;

  eprintln!("Scanning for atoms up to height {}…", tip_height);

  let mut atoms: BTreeMap<OutPoint, u64> = BTreeMap::new();

  for height in 0..tip_height {
    eprintln!("Scanning for atoms in block {}…", height);
    let hash = client.get_block_hash(height)?;

    let block = client.get_block(&hash)?;

    for (i, transaction) in block.txdata.iter().enumerate() {
      if i == 0 {
        atoms.insert(
          OutPoint {
            txid: transaction.txid(),
            vout: 0,
          },
          height,
        );
      } else {
        let transferred = transaction
          .input
          .iter()
          .map(|txin| atoms.remove(&txin.previous_output))
          .flatten()
          .collect::<Vec<u64>>();

        if !transferred.is_empty() {
          eprintln!(
            "Transferring {} atoms: {:?}",
            transferred.len(),
            transferred
          );
        }

        let value = transaction
          .output
          .iter()
          .map(|txout| txout.value)
          .sum::<u64>();

        for (vout, output) in transaction.output.iter().enumerate() {
          todo!("Transfer atoms proportionally according to output value");
        }
      }
    }
  }

  Ok(())
}
