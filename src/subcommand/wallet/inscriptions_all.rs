use {super::*, crate::wallet::Wallet};
use std::str;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub inscription: InscriptionId,
  pub location: SatPoint,
  pub explorer: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Brc20Deploy {
    pub p: String,
    pub op: String,
    pub tick: String,
    pub max: String,
    pub lim: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Brc20MintTransfer {
    pub p: String,
    pub op: String,
    pub tick: String,
    pub amt: String,
}

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
//   index.update()?;

  let inscriptions = index.get_inscriptions(None)?;
  let unspent_outputs = index.get_unspent_outputs(Wallet::load(&options)?)?;

  let explorer = match options.chain() {
    Chain::Mainnet => "https://ordinals.com/inscription/",
    Chain::Regtest => "http://localhost/inscription/",
    Chain::Signet => "https://signet.ordinals.com/inscription/",
    Chain::Testnet => "https://testnet.ordinals.com/inscription/",
  };

  let mut output = Vec::new();

  for (location, inscription) in inscriptions {
    let i = index.get_inscription_by_id(inscription).unwrap();

    // TODO: clean/rustify 
    if let Some(ct) = i.clone().unwrap().content_type() {
        if ct == "application/json" || ct == "text/plain;charset=utf-8" {
            if let Some(inc) = i.clone().unwrap().body() {
                match str::from_utf8(inc) {
                    Ok(parse_inc) => {
                        let deploy: Result<Brc20Deploy, _> = serde_json::from_str(parse_inc);
                        match deploy {
                            Ok(deploy) => {
                                println!("Deploy: {:?}", deploy);
                            }
                            Err(_) => {
                                let mint_transfer : Result<Brc20MintTransfer, _> = serde_json::from_str(parse_inc);
                                match mint_transfer {
                                    Ok(mint_transfer) => {
                                        println!("MintTransfer: {:?}", mint_transfer);
                                    }
                                    Err(_) => {
                                        // eprintln!("Failed to deserialize JSON: {}", &str::from_utf8(inc).unwrap());
                                    }
                                }
                            }
                        }
                    },
                    Err(_) => {},
                };
            }
        }
    }

    output.push(Output {
        location,
        inscription,
        explorer: format!("{explorer}{inscription}"),
      });
  }



//   print_json(&output)?;

  Ok(())
}
