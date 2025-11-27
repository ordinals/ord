use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
  pub cardinal: u64,
  pub ordinal: u64,
  pub total: u64,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let unspent_outputs = wallet.utxos();

  let inscription_outputs = wallet
    .inscriptions()
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let mut cardinal = 0;
  let mut ordinal = 0;

  for (output, txout) in unspent_outputs {
    let is_ordinal = inscription_outputs.contains(output);

    if is_ordinal {
      ordinal += txout.value.to_sat();
    }

    if !is_ordinal {
      cardinal += txout.value.to_sat();
    }

    if is_ordinal {
      eprintln!("warning: output {output} contains both inscriptions and runes");
    }
  }

  Ok(Some(Box::new(Output {
    cardinal,
    ordinal,
    total: cardinal + ordinal,
  })))
}