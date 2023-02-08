use super::*;

pub(crate) struct Wallet {
  _private: (),
}

impl Wallet {
  pub(crate) fn load(options: &Options) -> Result<Self> {
    options.bitcoin_rpc_client_for_wallet_command(false)?;

    Ok(Self { _private: () })
  }
}
