use {
  super::*,
  crate::runes::{varint, Edict, Runestone},
};

struct Allocation {
  balance: u128,
  divisibility: u8,
  id: u128,
  rune: Rune,
  symbol: Option<char>,
}

pub(super) struct RuneUpdater<'a, 'db, 'tx> {
  height: u64,
  id_to_entry: &'a mut Table<'db, 'tx, RuneIdValue, RuneEntryValue>,
  minimum: Rune,
  outpoint_to_balances: &'a mut Table<'db, 'tx, &'static OutPointValue, &'static [u8]>,
  rune_to_id: &'a mut Table<'db, 'tx, u128, RuneIdValue>,
}

impl<'a, 'db, 'tx> RuneUpdater<'a, 'db, 'tx> {
  pub(super) fn new(
    height: u64,
    outpoint_to_balances: &'a mut Table<'db, 'tx, &'static OutPointValue, &'static [u8]>,
    id_to_entry: &'a mut Table<'db, 'tx, RuneIdValue, RuneEntryValue>,
    rune_to_id: &'a mut Table<'db, 'tx, u128, RuneIdValue>,
  ) -> Self {
    Self {
      height,
      id_to_entry,
      minimum: Rune::minimum_at_height(Height(height)),
      outpoint_to_balances,
      rune_to_id,
    }
  }

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
            // Construct an allocation, representing the new runes that may be
            // allocated. Beware: Because it would require constructing a block
            // with 2**16 + 1 transactions, there is no test that checks that
            // an eching in a transaction with an out-of-bounds index is
            // ignored.
            match u16::try_from(index) {
              Ok(index) => Some(Allocation {
                balance: u128::max_value(),
                divisibility: etching.divisibility,
                id: u128::from(self.height) << 16 | u128::from(index),
                rune: etching.rune,
                symbol: etching.symbol,
              }),
              Err(_) => None,
            }
          }
        }
        None => None,
      };

      if !burn {
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
      }

      if let Some(Allocation {
        balance,
        divisibility,
        id,
        rune,
        symbol,
      }) = allocation
      {
        let id = RuneId::try_from(id).unwrap();
        self.rune_to_id.insert(rune.0, id.store())?;
        self.id_to_entry.insert(
          id.store(),
          RuneEntry {
            burned: 0,
            divisibility,
            etching: txid,
            rune,
            supply: u128::max_value() - balance,
            symbol,
          }
          .store(),
        )?;
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
