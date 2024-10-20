use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub transaction: Txid,
  pub confirmations: i32,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let unspent_outputs = wallet.utxos();

  let mut addresses: BTreeMap<Address, api::AddressInfo> = BTreeMap::new();

  let inscriptions = wallet
    .inscriptions()
    .clone()
    .into_iter()
    .map(|(satpoint, inscriptions)| (satpoint.outpoint, inscriptions))
    .collect::<BTreeMap<OutPoint, Vec<_>>>();

  for (output, txout) in unspent_outputs {
    let address = wallet.chain().address_from_script(&txout.script_pubkey)?;
    let rune_balances = wallet.get_runes_balances_in_output(output)?;

    addresses
      .entry(address)
      .and_modify(|info: &mut api::AddressInfo| {
        info.outputs.push(*output);
        info.inscriptions.extend(inscriptions.get(output).unwrap());
        info.sat_balance += txout.value.to_sat();
        info.runes_balances.extend(
          rune_balances
            .iter()
            .map(|(rune, pile)| {
              (
                *rune,
                Decimal {
                  value: pile.amount,
                  scale: pile.divisibility,
                },
                pile.symbol,
              )
            })
            .collect::<Vec<_>>(),
        );
      })
      .or_insert(api::AddressInfo {
        outputs: vec![*output],
        inscriptions: inscriptions.get(output).unwrap().clone(),
        sat_balance: txout.value.to_sat(),
        runes_balances: rune_balances
          .iter()
          .map(|(rune, pile)| {
            (
              *rune,
              Decimal {
                value: pile.amount,
                scale: pile.divisibility,
              },
              pile.symbol,
            )
          })
          .collect(),
      });
  }

  Ok(Some(Box::new(addresses)))
}
