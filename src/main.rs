use bitcoincore_rpc::{Auth, Client, RpcApi};

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

  let best_block_hash = client.get_best_block_hash().unwrap();
  println!("best block hash: {}", best_block_hash);

  Ok(())
}
