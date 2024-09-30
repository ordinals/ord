use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Outputs {
  #[arg(short, long, help = "Show list of sat <RANGES> in outputs.")]
  ranges: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sat_ranges: Option<Vec<String>>,
}

pub(super) async fn run(
  Extension(wallet): Extension<Arc<Mutex<Option<Arc<Wallet>>>>>,
  Extension(settings): Extension<Arc<Settings>>,
) -> ServerResult {
  let wallet = match init_wallet::init(wallet, settings).await {
    Ok(wallet) => wallet,
    Err(err) => {
        println!("Failed to initialize wallet: {:?}", err);
        return Err(anyhow!("Failed to initialize wallet").into());
    }
  };

  let mut outputs = Vec::new();
  for (output, txout) in wallet.utxos() {
    let sat_ranges = if wallet.has_sat_index() {
      Some(
        wallet
          .get_output_sat_ranges(output)?
          .into_iter()
          .map(|(start, end)| format!("{start}-{end}"))
          .collect(),
      )
    } else {
      None
    };

    outputs.push(Output {
      output: *output,
      amount: txout.value,
      sat_ranges,
    });
  }

  Ok(Json(outputs).into_response())
}
