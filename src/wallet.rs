use super::*;

pub(crate) struct Wallet {
  _private: (),
}

impl Wallet {
  pub(crate) fn load(options: &Options) -> Result<Self> {
    println!("BEFOREWALLET: {}", opt.wallet);
    options.bitcoin_rpc_client_for_wallet_command(false)?;
    println!("AFTerWALLET: {}", opt.wallet);
    Ok(Self { _private: () })
  }
}
