use {
  super::*,
  base64::{engine::general_purpose, Engine},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub address: Address<NetworkUnchecked>,
  pub witness: String,
}

#[derive(Debug, Parser)]
#[clap(
group(
  ArgGroup::new("input")
    .required(true)
    .args(&["text", "file"])))
]
pub(crate) struct Sign {
  #[arg(
    long,
    help = "Sign with public key associated with address, output, or inscription."
  )]
  signer: Signer,
  #[arg(long, help = "Sign <TEXT>.")]
  text: Option<String>,
  #[arg(long, help = "Sign contents of <FILE>.")]
  file: Option<PathBuf>,
}

impl Sign {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let address = match &self.signer {
      Signer::Address(address) => address.clone().require_network(wallet.chain().network())?,
      Signer::Inscription(inscription) => Address::from_str(
        &wallet
          .inscription_info()
          .get(inscription)
          .ok_or_else(|| anyhow!("inscription {inscription} not in wallet"))?
          .address
          .clone()
          .ok_or_else(|| anyhow!("inscription {inscription} in output without address"))?,
      )?
      .require_network(wallet.chain().network())?,
      Signer::Output(output) => wallet.chain().address_from_script(
        &wallet
          .utxos()
          .get(output)
          .ok_or_else(|| anyhow!("output {output} has no address"))?
          .script_pubkey,
      )?,
    };

    let message = if let Some(text) = &self.text {
      text.as_bytes()
    } else if let Some(file) = &self.file {
      &fs::read(file)?
    } else {
      unreachable!()
    };

    let to_spend = bip322::create_to_spend(&address, message)?;

    let to_sign = bip322::create_to_sign(&to_spend, None)?;

    let result = wallet.bitcoin_client().sign_raw_transaction_with_wallet(
      &to_sign.extract_tx()?,
      Some(&[bitcoincore_rpc::json::SignRawTransactionInput {
        txid: to_spend.compute_txid(),
        vout: 0,
        script_pub_key: address.script_pubkey(),
        redeem_script: None,
        amount: Some(Amount::ZERO),
      }]),
      None,
    )?;

    let mut buffer = Vec::new();

    Transaction::consensus_decode(&mut result.hex.as_slice())?.input[0]
      .witness
      .consensus_encode(&mut buffer)?;

    Ok(Some(Box::new(Output {
      address: address.as_unchecked().clone(),
      witness: general_purpose::STANDARD.encode(buffer),
    })))
  }
}
