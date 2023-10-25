use super::*;
use crate::{runes::*, subcommand::wallet::get_change_address, wallet::Wallet};
use bitcoin::blockdata::locktime::absolute::LockTime;

#[derive(Debug, Parser)]
pub(crate) struct Issue {
  #[arg(long, help = "Amount to cast")]
  pub amount: u128,
  #[arg(long, help = "Rune name")]
  pub name: String,
  #[arg(long, help = "Divisibility")]
  pub divisibility: u8,
}

impl Issue {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    if options.chain_argument == chain::Chain::Mainnet {
      return Ok(Box::new(
        "Runes are not yet available on mainnet, try running ord with '--chain signet'",
      ));
    }

    let runestone = Runestone {
      edicts: vec![Edict {
        id: 0,
        amount: self.amount,
        output: 1,
      }],
      etching: Some(Etching {
        divisibility: self.divisibility,
        rune: crate::runes::Rune::from_str(&self.name)?,
        symbol: None,
      }),
      burn: false,
    };

    let index = Index::open(&options)?;
    index.update()?;

    let utxos = index.get_unspent_outputs(Wallet::load(&options)?)?;
    let inscriptions = index.get_inscriptions(&utxos)?;

    let inscribed_utxos = inscriptions
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<std::collections::BTreeSet<OutPoint>>();

    let chain = options.chain();

    let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let cardinal_utxos = utxos
      .iter()
      .filter_map(|e| match inscribed_utxos.get(e.0) {
        Some(_) => None,
        None => Some(e.clone()),
      })
      .filter_map(
        |e| match index.get_rune_balances_for_outpoint(e.0.clone()) {
          Ok(v) => {
            if v.len() > 0 {
              return None;
            }

            Some(e.clone())
          }
          Err(_) => None,
        },
      )
      .collect::<Vec<_>>();

    let utxo = &cardinal_utxos[0];

    let tx = Transaction {
      version: 1,
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: utxo.0.clone(),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: bitcoin::blockdata::witness::Witness::new(),
      }],
      output: vec![
        TxOut {
          value: 0,
          script_pubkey: runestone.encipher(),
        },
        TxOut {
          value: 546,
          script_pubkey: get_change_address(&client, chain)?.script_pubkey(),
        },
        TxOut {
          value: utxo.1.to_sat() - 1000,
          script_pubkey: get_change_address(&client, chain)?.script_pubkey(),
        },
      ],
    };

    let signed_tx = client
      .sign_raw_transaction_with_wallet(&tx, None, None)?
      .hex;
    let txid = client.send_raw_transaction(&signed_tx)?;

    Ok(Box::new(txid))
  }
}
