use crate::index::entry::Entry;
use crate::index::{InscriptionIdValue, TxidValue};
use crate::inscription_id::InscriptionId;
use crate::okx::datastore::brc20::redb::{
  max_script_tick_id_key, max_script_tick_key, min_script_tick_id_key, min_script_tick_key,
  script_tick_id_key, script_tick_key,
};
use crate::okx::datastore::brc20::{
  Balance, Receipt, Tick, TokenInfo, TransferInfo, TransferableLog,
};
use crate::okx::datastore::ScriptKey;
use bitcoin::Txid;
use redb::{MultimapTable, ReadableMultimapTable, ReadableTable, Table};

// BRC20_BALANCES
pub fn get_balances<T>(table: &T, script_key: &ScriptKey) -> crate::Result<Vec<Balance>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .range(min_script_tick_key(script_key).as_str()..=max_script_tick_key(script_key).as_str())?
      .flat_map(|result| {
        result.map(|(_, data)| rmp_serde::from_slice::<Balance>(data.value()).unwrap())
      })
      .collect(),
  )
}

// BRC20_BALANCES
pub fn get_balance<T>(
  table: &T,
  script_key: &ScriptKey,
  tick: &Tick,
) -> crate::Result<Option<Balance>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .get(script_tick_key(script_key, tick).as_str())?
      .map(|v| rmp_serde::from_slice::<Balance>(v.value()).unwrap()),
  )
}

// BRC20_TOKEN
pub fn get_token_info<T>(table: &T, tick: &Tick) -> crate::Result<Option<TokenInfo>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .get(tick.to_lowercase().hex().as_str())?
      .map(|v| rmp_serde::from_slice::<TokenInfo>(v.value()).unwrap()),
  )
}

// BRC20_TOKEN
pub fn get_tokens_info<T>(table: &T) -> crate::Result<Vec<TokenInfo>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .range::<&str>(..)?
      .flat_map(|result| {
        result.map(|(_, data)| rmp_serde::from_slice::<TokenInfo>(data.value()).unwrap())
      })
      .collect(),
  )
}

// BRC20_EVENTS
pub fn get_transaction_receipts<T>(table: &T, txid: &Txid) -> crate::Result<Vec<Receipt>>
where
  T: ReadableMultimapTable<&'static TxidValue, &'static [u8]>,
{
  Ok(
    table
      .get(&txid.store())?
      .into_iter()
      .map(|x| rmp_serde::from_slice::<Receipt>(x.unwrap().value()).unwrap())
      .collect(),
  )
}

// BRC20_TRANSFERABLELOG
pub fn get_transferable<T>(table: &T, script: &ScriptKey) -> crate::Result<Vec<TransferableLog>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .range(min_script_tick_key(script).as_str()..max_script_tick_key(script).as_str())?
      .flat_map(|result| {
        result.map(|(_, v)| rmp_serde::from_slice::<Vec<TransferableLog>>(v.value()).unwrap())
      })
      .flatten()
      .collect(),
  )
}

// BRC20_TRANSFERABLELOG
pub fn get_transferable_by_tick<T>(
  table: &T,
  script: &ScriptKey,
  tick: &Tick,
) -> crate::Result<Vec<TransferableLog>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    table
      .range(
        min_script_tick_id_key(script, tick).as_str()
          ..max_script_tick_id_key(script, tick).as_str(),
      )?
      .flat_map(|result| {
        result.map(|(_, v)| rmp_serde::from_slice::<Vec<TransferableLog>>(v.value()).unwrap())
      })
      .flatten()
      .collect(),
  )
}

// BRC20_TRANSFERABLELOG
pub fn get_transferable_by_id<T>(
  table: &T,
  script: &ScriptKey,
  inscription_id: &InscriptionId,
) -> crate::Result<Option<TransferableLog>>
where
  T: ReadableTable<&'static str, &'static [u8]>,
{
  Ok(
    get_transferable(table, script)?
      .iter()
      .find(|log| log.inscription_id == *inscription_id)
      .cloned(),
  )
}

// BRC20_INSCRIBE_TRANSFER
pub fn get_inscribe_transfer_inscription<T>(
  table: &T,
  inscription_id: &InscriptionId,
) -> crate::Result<Option<TransferInfo>>
where
  T: ReadableTable<InscriptionIdValue, &'static [u8]>,
{
  Ok(
    table
      .get(&inscription_id.store())?
      .map(|v| rmp_serde::from_slice::<TransferInfo>(v.value()).unwrap()),
  )
}

// BRC20_BALANCES
pub fn update_token_balance<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  script_key: &ScriptKey,
  new_balance: Balance,
) -> crate::Result<()> {
  table.insert(
    script_tick_key(script_key, &new_balance.tick).as_str(),
    rmp_serde::to_vec(&new_balance).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_TOKEN
pub fn insert_token_info<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  tick: &Tick,
  new_info: &TokenInfo,
) -> crate::Result<()> {
  table.insert(
    tick.to_lowercase().hex().as_str(),
    rmp_serde::to_vec(new_info).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_TOKEN
pub fn update_mint_token_info<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  tick: &Tick,
  minted_amt: u128,
  minted_block_number: u32,
) -> crate::Result<()> {
  let mut info =
    get_token_info(table, tick)?.unwrap_or_else(|| panic!("token {} not exist", tick.as_str()));

  info.minted = minted_amt;
  info.latest_mint_number = minted_block_number;

  table.insert(
    tick.to_lowercase().hex().as_str(),
    rmp_serde::to_vec(&info).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_EVENTS
pub fn add_transaction_receipt<'db, 'txn>(
  table: &mut MultimapTable<'db, 'txn, &'static TxidValue, &'static [u8]>,
  txid: &Txid,
  receipt: &Receipt,
) -> crate::Result<()> {
  table.insert(
    &txid.store(),
    rmp_serde::to_vec(receipt).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_TRANSFERABLELOG
pub fn insert_transferable<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  script: &ScriptKey,
  tick: &Tick,
  inscription: &TransferableLog,
) -> crate::Result<()> {
  table.insert(
    script_tick_id_key(script, tick, &inscription.inscription_id).as_str(),
    rmp_serde::to_vec(&inscription).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_TRANSFERABLELOG
pub fn remove_transferable<'db, 'txn>(
  table: &mut Table<'db, 'txn, &'static str, &'static [u8]>,
  script: &ScriptKey,
  tick: &Tick,
  inscription_id: &InscriptionId,
) -> crate::Result<()> {
  table.remove(script_tick_id_key(script, tick, inscription_id).as_str())?;
  Ok(())
}

// BRC20_INSCRIBE_TRANSFER
pub fn insert_inscribe_transfer_inscription<'db, 'txn>(
  table: &mut Table<'db, 'txn, InscriptionIdValue, &'static [u8]>,
  inscription_id: &InscriptionId,
  transfer_info: TransferInfo,
) -> crate::Result<()> {
  table.insert(
    &inscription_id.store(),
    rmp_serde::to_vec(&transfer_info).unwrap().as_slice(),
  )?;
  Ok(())
}

// BRC20_INSCRIBE_TRANSFER
pub fn remove_inscribe_transfer_inscription<'db, 'txn>(
  table: &mut Table<'db, 'txn, InscriptionIdValue, &'static [u8]>,
  inscription_id: &InscriptionId,
) -> crate::Result<()> {
  table.remove(&inscription_id.store())?;
  Ok(())
}
