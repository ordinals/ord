use super::*;

pub fn initialize() -> Result<Client> {
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

  Ok(Client::new(
    "http://localhost:8332",
    Auth::CookieFile(cookiefile),
  )?)
}
