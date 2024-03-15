use {
  super::*,
  bitcoin::{
    bip32::{DerivationPath, ExtendedPrivKey},
    secp256k1::{All, Secp256k1},
    Network, PrivateKey, PublicKey,
  },
  miniscript::ToPublicKey,
};

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub(crate) enum SelectNetwork {
  Mainnet,
  Testnet,
  Signet,
  Regtest,
}

impl From<SelectNetwork> for Network {
  fn from(network: SelectNetwork) -> Network {
    match network {
      SelectNetwork::Mainnet => Network::Bitcoin,
      SelectNetwork::Testnet => Network::Testnet,
      SelectNetwork::Signet => Network::Signet,
      SelectNetwork::Regtest => Network::Regtest,
    }
  }
}

#[derive(Debug, Parser)]
pub(crate) struct Generate {
  #[arg(
    short,
    long,
    default_value = "",
    help = "Use <PASSPHRASE> to derive wallet seed."
  )]
  pub(crate) passphrase: String,
  #[arg(
    short,
    long,
    default_value_t = 12,
    help = "The number of words in the phrase to generate"
  )]
  pub(crate) words: usize,
  #[clap(value_enum, short, long, default_value_t = SelectNetwork::Mainnet, help = "Specify network")]
  pub(crate) network: SelectNetwork,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AddressInfo {
  pub address: String,
  pub path: String,
  #[serde(rename(serialize = "WIF"))]
  pub(crate) wif: String,
  pub(crate) public_key: PublicKey,
  pub(crate) private_key: PrivateKey,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Wallet {
  #[serde(rename(serialize = "Native Segwit(P2WPKH)"))]
  pub p2wpkh: AddressInfo,
  #[serde(rename(serialize = "Nested Segwit(P2SH-P2WPKH)"))]
  pub p2shp2wpkh: AddressInfo,
  #[serde(rename(serialize = "Taproot(P2TR)"))]
  pub p2tr: AddressInfo,
  #[serde(rename(serialize = "Legacy(P2PKH)"))]
  pub p2pkh: AddressInfo,
}

impl Wallet {
  pub(crate) fn create(seed: &[u8], network: Network) -> Result<Wallet> {
    let secp = Secp256k1::new();
    let master_key = ExtendedPrivKey::new_master(network, seed)?;
    Ok(Wallet {
      p2wpkh: Self::generate_address_info(&secp, "m/84'/0'/0'/0/0", network, &master_key)?,
      p2shp2wpkh: Self::generate_address_info(&secp, "m/49'/0'/0'/0/0", network, &master_key)?,
      p2tr: Self::generate_address_info(&secp, "m/86'/0'/0'/0/0", network, &master_key)?,
      p2pkh: Self::generate_address_info(&secp, "m/44'/0'/0'/0/0", network, &master_key)?,
    })
  }

  fn generate_address_info(
    secp: &Secp256k1<All>,
    derivation_path: &str,
    network: Network,
    master_key: &ExtendedPrivKey,
  ) -> Result<AddressInfo> {
    let path = DerivationPath::from_str(derivation_path)?;
    let derived_key = master_key.derive_priv(secp, &path)?;
    let key_pair = derived_key.to_keypair(secp);
    let private_key = PrivateKey::new(key_pair.secret_key(), network);
    let public_key = PublicKey::from_private_key(secp, &private_key);
    let address = match derivation_path {
      "m/44'/0'/0'/0/0" => Address::p2pkh(&public_key.to_public_key(), network).to_string(),
      "m/49'/0'/0'/0/0" => {
        let p2wpkh = Address::p2wpkh(&public_key.to_public_key(), network)?;
        let p2sh_script = p2wpkh.script_pubkey();
        Address::p2sh(&p2sh_script, network)?.to_string()
      }
      "m/84'/0'/0'/0/0" => Address::p2wpkh(&public_key.to_public_key(), network)?.to_string(),
      _ => Address::p2tr(secp, public_key.to_x_only_pubkey(), None, network).to_string(),
    };
    let wif = private_key.to_wif();
    Ok(AddressInfo {
      address,
      path: derivation_path.to_string(),
      wif,
      public_key,
      private_key,
    })
  }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Output {
  pub(crate) mnemonic: Mnemonic,
  pub(crate) passphrase: String,
  pub(crate) wallet: Wallet,
}

impl Generate {
  pub(crate) fn run(&self) -> SubcommandResult {
    ensure!(
      [12, 15, 18, 21, 24].contains(&self.words),
      "BIP-39 specification, mnemonics are usually composed of 12, 15, 18, 21, or 24 words"
    );
    let mnemonic = Mnemonic::generate(self.words)?;
    let seed = mnemonic.to_seed(&self.passphrase);
    let wallet = Wallet::create(&seed, self.network.into())?;
    Ok(Some(Box::new(Output {
      mnemonic,
      passphrase: self.passphrase.clone(),
      wallet,
    })))
  }
}
