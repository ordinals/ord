use super::*;

mod inscription;
mod outpoint;
mod transaction;

use crate::index::{Flotsam, Origin};
use crate::okx::datastore::ord::{Action, InscriptionOp};
pub(super) use {inscription::*, outpoint::*, transaction::*};

#[derive(Debug, thiserror::Error)]
pub enum OrdError {
  #[error("operation not found")]
  OperationNotFound,
  #[error("block not found")]
  BlockNotFound,
}

pub(super) fn get_ord_operations_by_txid(
  index: &Arc<Index>,
  txid: &bitcoin::Txid,
  with_unconfirmed: bool,
) -> Result<Vec<InscriptionOp>> {
  let tx = index
    .get_transaction_info(txid)?
    .ok_or(anyhow!("can't get transaction info: {txid}"))?;

  match tx.confirmations {
    None => {
      if with_unconfirmed {
        // If the transaction is not confirmed, simulate indexing the transaction. Otherwise, retrieve it from the database.
        simulate_index_ord_transaction(index, &tx.transaction()?, tx.txid)
      } else {
        Err(anyhow!("transaction not confirmed: {txid}"))
      }
    }
    // TODO: retrieve it from the database.
    Some(_) => Err(anyhow!("not implemented")),
  }
}

/// Simulate the execution of a transaction and parse out the inscription operation.
fn simulate_index_ord_transaction(
  index: &Arc<Index>,
  tx: &Transaction,
  txid: Txid,
) -> Result<Vec<InscriptionOp>> {
  let mut operations = Vec::new();
  let mut envelopes = ParsedEnvelope::from_transaction(tx).into_iter().peekable();
  let mut floating_inscriptions = Vec::new();
  let mut id_counter = 0;
  let mut inscribed_offsets = BTreeMap::new();
  let mut total_input_value = 0;
  let total_output_value = tx.output.iter().map(|txout| txout.value).sum::<u64>();

  for (input_index, tx_in) in tx.input.iter().enumerate() {
    // skip subsidy since no inscriptions possible
    if tx_in.previous_output.is_null() {
      return Ok(operations);
    }

    // find existing inscriptions on input (transfers of inscriptions)
    for (old_satpoint, inscription_id) in
      index.get_inscriptions_with_satpoint_on_output(tx_in.previous_output)?
    {
      let offset = total_input_value + old_satpoint.offset;
      floating_inscriptions.push(Flotsam {
        txid,
        offset,
        inscription_id,
        old_satpoint,
        origin: Origin::Old,
      });
    }

    let offset = total_input_value;

    // multi-level cache for UTXO set to get to the input amount
    let current_input_value =
      if let Some(tx_out) = index.get_transaction_output_by_outpoint(tx_in.previous_output)? {
        tx_out.value
      } else if let Some(tx) = index.get_transaction_with_retries(tx_in.previous_output.txid)? {
        tx.output
          .get(tx_in.previous_output.vout as usize)
          .unwrap()
          .value
      } else {
        return Err(anyhow!(
          "can't get transaction output by outpoint: {}",
          tx_in.previous_output
        ));
      };

    total_input_value += current_input_value;

    // go through all inscriptions in this input
    while let Some(inscription) = envelopes.peek() {
      if inscription.input != u32::try_from(input_index).unwrap() {
        break;
      }

      let inscription_id = InscriptionId {
        txid,
        index: id_counter,
      };

      let cursed = false;

      // assume height has passed jubilee height and there is no cursed inscription
      let unbound = current_input_value == 0;

      let offset = inscription
        .payload
        .pointer()
        .filter(|&pointer| pointer < total_output_value)
        .unwrap_or(offset);

      floating_inscriptions.push(Flotsam {
        txid,
        inscription_id,
        offset,
        old_satpoint: SatPoint {
          outpoint: tx_in.previous_output,
          offset: 0,
        },
        origin: Origin::New {
          reinscription: inscribed_offsets.get(&offset).is_some(),
          cursed,
          fee: 0,
          hidden: inscription.payload.hidden(),
          parent: inscription.payload.parent(),
          pointer: inscription.payload.pointer(),
          unbound,
          inscription: inscription.payload.clone(),
        },
      });

      inscribed_offsets
        .entry(offset)
        .or_insert((inscription_id, 0))
        .1 += 1;

      envelopes.next();
      id_counter += 1;
    }
  }

  let potential_parents = floating_inscriptions
    .iter()
    .map(|flotsam| flotsam.inscription_id)
    .collect::<HashSet<InscriptionId>>();

  for flotsam in &mut floating_inscriptions {
    if let Flotsam {
      origin: Origin::New { parent, .. },
      ..
    } = flotsam
    {
      if let Some(purported_parent) = parent {
        if !potential_parents.contains(purported_parent) {
          *parent = None;
        }
      }
    }
  }

  // still have to normalize over inscription size
  for flotsam in &mut floating_inscriptions {
    if let Flotsam {
      origin: Origin::New { ref mut fee, .. },
      ..
    } = flotsam
    {
      *fee = (total_input_value - total_output_value) / u64::from(id_counter);
    }
  }

  floating_inscriptions.sort_by_key(|flotsam| flotsam.offset);
  let mut inscriptions = floating_inscriptions.into_iter().peekable();

  let mut new_locations = Vec::new();
  let mut output_value = 0;
  for (vout, tx_out) in tx.output.iter().enumerate() {
    let end = output_value + tx_out.value;

    while let Some(flotsam) = inscriptions.peek() {
      if flotsam.offset >= end {
        break;
      }

      let new_satpoint = SatPoint {
        outpoint: OutPoint {
          txid,
          vout: vout.try_into().unwrap(),
        },
        offset: flotsam.offset - output_value,
      };

      new_locations.push((new_satpoint, inscriptions.next().unwrap()));
    }

    output_value = end;
  }

  // Inscription not found with matching output position.
  operations.extend(inscriptions.map(|flotsam| InscriptionOp {
    txid: flotsam.txid,
    // We use 0 to represent the default sequence_number.
    sequence_number: 0,
    inscription_number: None,
    inscription_id: flotsam.inscription_id,
    action: match flotsam.origin {
      Origin::Old => Action::Transfer,
      Origin::New {
        cursed,
        fee: _,
        hidden: _,
        parent: _,
        pointer: _,
        reinscription: _,
        unbound,
        inscription,
      } => Action::New {
        cursed,
        unbound,
        inscription,
      },
    },
    old_satpoint: flotsam.old_satpoint,
    // We use a zero satpoint to represent the default position.
    new_satpoint: None,
  }));

  Ok(operations)
}
