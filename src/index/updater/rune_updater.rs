use {
  super::*,
  crate::runes::{varint, Edict, Runestone, CLAIM_BIT},
};

fn claim(id: u128) -> Option<u128> {
  (id & CLAIM_BIT != 0).then_some(id ^ CLAIM_BIT)
}

struct Allocation {
  balance: u128,
  divisibility: u8,
  end: Option<u32>,
  id: u128,
  limit: Option<u128>,
  rune: Rune,
  symbol: Option<char>,
}

pub(super) struct RuneUpdater<'a, 'db, 'tx> {
  pub(super) height: u32,
  pub(super) id_to_entry: &'a mut Table<'db, 'tx, RuneIdValue, RuneEntryValue>,
  pub(super) inscription_id_to_sequence_number: &'a Table<'db, 'tx, InscriptionIdValue, u32>,
  pub(super) minimum: Rune,
  pub(super) outpoint_to_balances: &'a mut Table<'db, 'tx, &'static OutPointValue, &'static [u8]>,
  pub(super) rune_to_id: &'a mut Table<'db, 'tx, u128, RuneIdValue>,
  pub(super) runes: u64,
  pub(super) sequence_number_to_rune: &'a mut Table<'db, 'tx, u32, u128>,
  pub(super) statistic_to_count: &'a mut Table<'db, 'tx, u64, u64>,
  pub(super) timestamp: u32,
  pub(super) transaction_id_to_rune: &'a mut Table<'db, 'tx, &'static TxidValue, u128>,
}

impl<'a, 'db, 'tx> RuneUpdater<'a, 'db, 'tx> {
  pub(super) fn index_runes(&mut self, index: usize, tx: &Transaction, txid: Txid) -> Result<()> {
    let runestone = Runestone::from_transaction(tx);

    // A mapping of rune ID to un-allocated balance of that rune
    let mut unallocated: HashMap<u128, u128> = HashMap::new();

    // Increment unallocated runes with the runes in this transaction's inputs
    for input in &tx.input {
      if let Some(guard) = self
        .outpoint_to_balances
        .remove(&input.previous_output.store())?
      {
        let buffer = guard.value();
        let mut i = 0;
        while i < buffer.len() {
          let (id, len) = varint::decode(&buffer[i..])?;
          i += len;
          let (balance, len) = varint::decode(&buffer[i..])?;
          i += len;
          *unallocated.entry(id).or_default() += balance;
        }
      }
    }

    let burn = runestone
      .as_ref()
      .map(|runestone| runestone.burn)
      .unwrap_or_default();

    // A vector of allocated transaction output rune balances
    let mut allocated: Vec<HashMap<u128, u128>> = vec![HashMap::new(); tx.output.len()];

    if let Some(runestone) = runestone {
      // Determine if this runestone conains a valid issuance
      let mut allocation = match runestone.etching {
        Some(etching) => {
          // If the issuance symbol is already taken, the issuance is ignored
          if etching.rune < self.minimum || self.rune_to_id.get(etching.rune.0)?.is_some() {
            None
          } else {
            let (limit, term) = match (etching.limit, etching.term) {
              (None, Some(term)) => (Some(runes::MAX_LIMIT), Some(term)),
              (limit, term) => (limit, term),
            };

            // Construct an allocation, representing the new runes that may be
            // allocated. Beware: Because it would require constructing a block
            // with 2**16 + 1 transactions, there is no test that checks that
            // an eching in a transaction with an out-of-bounds index is
            // ignored.
            match u16::try_from(index) {
              Ok(index) => Some(Allocation {
                balance: if let Some(limit) = limit {
                  if term == Some(0) {
                    0
                  } else {
                    limit
                  }
                } else {
                  u128::max_value()
                },
                limit,
                divisibility: etching.divisibility,
                id: u128::from(self.height) << 16 | u128::from(index),
                rune: etching.rune,
                symbol: etching.symbol,
                end: term.map(|term| term + self.height),
              }),
              Err(_) => None,
            }
          }
        }
        None => None,
      };

      if !burn {
        let mut mintable: HashMap<u128, u128> = HashMap::new();

        let mut claims = runestone
          .edicts
          .iter()
          .filter_map(|edict| claim(edict.id))
          .collect::<Vec<u128>>();
        claims.sort();
        claims.dedup();
        for id in claims {
          if let Ok(key) = RuneId::try_from(id) {
            if let Some(entry) = self.id_to_entry.get(&key.store())? {
              let entry = RuneEntry::load(entry.value());
              if let Some(limit) = entry.limit {
                if let Some(end) = entry.end {
                  if self.height >= end {
                    continue;
                  }
                }
                mintable.insert(id, limit);
              }
            }
          }
        }

        let limits = mintable.clone();

        for Edict { id, amount, output } in runestone.edicts {
          let Ok(output) = usize::try_from(output) else {
            continue;
          };

          // Skip edicts not referring to valid outputs
          if output > tx.output.len() {
            continue;
          }

          let (balance, id) = if id == 0 {
            // If this edict allocates new issuance runes, skip it
            // if no issuance was present, or if the issuance was invalid.
            // Additionally, replace ID 0 with the newly assigned ID, and
            // get the unallocated balance of the issuance.
            match allocation.as_mut() {
              Some(Allocation { balance, id, .. }) => (balance, *id),
              None => continue,
            }
          } else if let Some(claim) = claim(id) {
            match mintable.get_mut(&claim) {
              Some(balance) => (balance, claim),
              None => continue,
            }
          } else {
            // Get the unallocated balance of the given ID
            match unallocated.get_mut(&id) {
              Some(balance) => (balance, id),
              None => continue,
            }
          };

          let mut allocate = |balance: &mut u128, amount: u128, output: usize| {
            if amount > 0 {
              *balance -= amount;
              *allocated[output].entry(id).or_default() += amount;
            }
          };

          if output == tx.output.len() {
            // find non-OP_RETURN outputs
            let destinations = tx
              .output
              .iter()
              .enumerate()
              .filter_map(|(output, tx_out)| {
                (!tx_out.script_pubkey.is_op_return()).then_some(output)
              })
              .collect::<Vec<usize>>();

            if amount == 0 {
              // if amount is zero, divide balance between eligible outputs
              let amount = *balance / destinations.len() as u128;

              for output in destinations {
                allocate(balance, amount, output);
              }
            } else {
              // if amount is non-zero, distribute amount to eligible outputs
              for output in destinations {
                allocate(balance, amount.min(*balance), output);
              }
            }
          } else {
            // Get the allocatable amount
            let amount = if amount == 0 {
              *balance
            } else {
              amount.min(*balance)
            };

            allocate(balance, amount, output);
          }
        }

        // increment entries with minted runes
        for (id, amount) in mintable {
          let minted = limits[&id] - amount;
          if minted > 0 {
            let id = RuneId::try_from(id).unwrap().store();
            let mut entry = RuneEntry::load(self.id_to_entry.get(id)?.unwrap().value());
            entry.supply += minted;
            self.id_to_entry.insert(id, entry.store())?;
          }
        }
      }

      if let Some(Allocation {
        balance,
        divisibility,
        id,
        rune,
        symbol,
        limit,
        end,
      }) = allocation
      {
        let id = RuneId::try_from(id).unwrap();
        self.rune_to_id.insert(rune.0, id.store())?;
        self.transaction_id_to_rune.insert(&txid.store(), rune.0)?;
        let number = self.runes;
        self.runes += 1;
        self
          .statistic_to_count
          .insert(&Statistic::Runes.into(), self.runes)?;
        self.id_to_entry.insert(
          id.store(),
          RuneEntry {
            burned: 0,
            divisibility,
            etching: txid,
            number,
            rune,
            supply: if let Some(limit) = limit {
              if end == Some(self.height) {
                0
              } else {
                limit
              }
            } else {
              u128::max_value()
            } - balance,
            end,
            symbol,
            limit,
            timestamp: self.timestamp,
          }
          .store(),
        )?;

        let inscription_id = InscriptionId { txid, index: 0 };

        if let Some(sequence_number) = self
          .inscription_id_to_sequence_number
          .get(&inscription_id.store())?
        {
          self
            .sequence_number_to_rune
            .insert(sequence_number.value(), rune.0)?;
        }
      }
    }

    let mut burned: HashMap<u128, u128> = HashMap::new();

    if burn {
      for (id, balance) in unallocated {
        *burned.entry(id).or_default() += balance;
      }
    } else {
      // Assign all un-allocated runes to the first non OP_RETURN output
      if let Some((vout, _)) = tx
        .output
        .iter()
        .enumerate()
        .find(|(_, tx_out)| !tx_out.script_pubkey.is_op_return())
      {
        for (id, balance) in unallocated {
          if balance > 0 {
            *allocated[vout].entry(id).or_default() += balance;
          }
        }
      }
    }

    // update outpoint balances
    let mut buffer: Vec<u8> = Vec::new();
    for (vout, balances) in allocated.into_iter().enumerate() {
      if balances.is_empty() {
        continue;
      }

      // increment burned balances
      if tx.output[vout].script_pubkey.is_op_return() {
        for (id, balance) in &balances {
          *burned.entry(*id).or_default() += balance;
        }
        continue;
      }

      buffer.clear();

      let mut balances = balances.into_iter().collect::<Vec<(u128, u128)>>();

      // Sort balances by id so tests can assert balances in a fixed order
      balances.sort();

      for (id, balance) in balances {
        varint::encode_to_vec(id, &mut buffer);
        varint::encode_to_vec(balance, &mut buffer);
      }

      self.outpoint_to_balances.insert(
        &OutPoint {
          txid,
          vout: vout.try_into().unwrap(),
        }
        .store(),
        buffer.as_slice(),
      )?;
    }

    // increment entries with burned runes
    for (id, amount) in burned {
      let id = RuneId::try_from(id).unwrap().store();
      let mut entry = RuneEntry::load(self.id_to_entry.get(id)?.unwrap().value());
      entry.burned += amount;
      self.id_to_entry.insert(id, entry.store())?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn claim_from_id() {
    assert_eq!(claim(1), None);
    assert_eq!(claim(1 | CLAIM_BIT), Some(1));
  }
}
