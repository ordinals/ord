use {
  crate::index::entry::BridgeEntry,
  crate::outgoing::Outgoing,
  crate::subcommand::wallet::send,
  crate::wallet::Wallet,
  crate::*,
  base64::Engine,
  bitcoin::psbt::Psbt,
  bitcoin::secp256k1::ffi::CPtr,
  core::ffi::{c_char, c_int, c_uchar, c_void},
  std::ffi::CString,
};

// must stay in sync with `tap/tap.go`.
#[repr(C)]
pub struct CRuneEntry {
  block: u64,
  burned_low: u64,
  burned_high: u64,
  divisibility: u8,
  etching: *const c_uchar,
  mints_low: u64,
  mints_high: u64,
  number: u64,
  premine_low: u64,
  premine_high: u64,
  symbol: c_char,
  timestamp: u64,
  turbo: bool,
}

// must stay in sync with `tap/tap.go`.
#[repr(C)]
pub struct CRuneID {
  block: u64,
  tx: u32,
}

// must stay in sync with `tap/tap.go`.
#[repr(C)]
pub struct CTapLockConfig {
  transaction: *const c_void,
  transaction_length: c_int,
  rune_entry: *const CRuneEntry,
  amount: u64,
  script_key: *const u8,
  batch_key: *const u8,
  rune_id: *const CRuneID,
}

// must stay in sync with `tap/tap.go`.
#[repr(C)]
pub struct CTapLockResult {
  transaction: *const c_void,
  transaction_length: c_int,
  asset_id: *const c_char,
}

// must stay in sync with `tap/tap.go`.
#[repr(C)]
pub struct CProofConfig {
  packet: *const c_void,
  packet_length: c_int,
  block: *const c_void,
  block_length: c_int,
  transaction_index: u32,
  height: u32,
  amount: u64,
  script_key: *const u8,
  batch_key: *const u8,
  rune_id: *const CRuneID,
  universe_address: *const c_uchar,
}

extern "C" {
  fn TapLock(config: *const CTapLockConfig) -> *const CTapLockResult;
  fn TapPublishProof(config: *const CProofConfig);
}

// bitcoind doesn't have a direct way to derive a public key, so instead we create an address and query the public key.
fn derive_public_key(wallet: &Wallet, label: &str) -> Result<[u8; 33]> {
  let bitcoin_client = wallet.bitcoin_client();

  let address = bitcoin_client
    .get_new_address(
      Some(label),
      Some(bitcoincore_rpc::json::AddressType::Bech32),
    )?
    .assume_checked();

  let address = bitcoin_client.get_address_info(&address)?;

  if let Some(public_key) = address.pubkey {
    Ok(public_key.to_bytes().try_into().unwrap())
  } else {
    bail!("address info did not return a public key");
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub txid: Txid,
  pub asset_id: String,
  pub batch_key: String,
  pub script_key: String,
}

pub(crate) fn lock(wallet: Wallet, fee_rate: FeeRate, outgoing: Outgoing) -> Result<Output> {
  let Outgoing::Rune {
    decimal,
    rune: spaced_rune,
  } = outgoing
  else {
    todo!("`ord wallet bridge-lock` only supports runes");
  };

  let (id, entry, _parent) = wallet
    .get_rune(spaced_rune.rune)?
    .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;

  // convert the amount to a integer.
  let amount = decimal.to_integer(entry.divisibility)?;

  // ensure the amount fits in a taproot asset.
  if amount > u64::MAX as u128 {
    panic!("amount does not fit in uint64, unsupported by taproot assets");
  }

  let input = send::fund_runes_transaction(&wallet, entry, amount)?;

  let runestone = Runestone {
    bridge: Some(Bridge {
      id,
      amount,
      index: 2,
      lock: true,
    }),
    ..default()
  };

  let unfunded_transaction = Transaction {
    version: 2,
    lock_time: LockTime::ZERO,
    input,
    output: vec![
      TxOut {
        script_pubkey: runestone.encipher(),
        value: 0,
      },
      TxOut {
        script_pubkey: wallet.get_change_address()?.script_pubkey(),
        value: TARGET_POSTAGE.to_sat(),
      },
      // taproot asset output will be here.
    ],
  };

  let bitcoin_client = wallet.bitcoin_client();

  let mut raw_transaction =
    fund_raw_transaction(bitcoin_client, fee_rate, &unfunded_transaction)?.hex;
  // derive the script and the batch key for the taproot asset mint.
  let script_key = derive_public_key(&wallet, "taproot-assets-script-key")?;
  let batch_key = derive_public_key(&wallet, "taproot-assets-batch-key")?;

  let asset_id;

  // lock the transaction, add taproot assets input & output.
  unsafe {
    println!("result");

    let result = TapLock(&CTapLockConfig {
      transaction: raw_transaction.as_c_ptr() as *const c_void,
      transaction_length: raw_transaction.len() as c_int,
      rune_entry: &CRuneEntry {
        block: entry.block,

        // split into two u64s.
        burned_low: entry.burned as u64,
        burned_high: (entry.burned >> 64) as u64,

        divisibility: entry.divisibility,

        etching: entry.etching.to_byte_array().as_c_ptr(),

        // split into two u64s.
        mints_low: entry.mints as u64,
        mints_high: (entry.mints >> 64) as u64,

        number: entry.number,

        // split into two u64s.
        premine_low: entry.premine as u64,
        premine_high: (entry.premine >> 64) as u64,

        // unwrap or fallback to 0.
        symbol: entry.symbol.unwrap_or(0 as char) as c_char,

        timestamp: entry.timestamp,
        turbo: entry.turbo,
      },
      amount: amount as u64,
      script_key: script_key.as_c_ptr(),
      batch_key: batch_key.as_c_ptr(),
      rune_id: &CRuneID {
        block: id.block,
        tx: id.tx,
      },
    });

    raw_transaction = std::slice::from_raw_parts(
      (*result).transaction as *mut u8,
      (*result).transaction_length as usize,
    )
    .to_vec();

    asset_id = CString::from_raw((*result).asset_id as *mut i8).into_string()?;
  }

  // deserialize partially, since the vector has padding.
  let unsigned_transaction: Transaction = consensus::encode::deserialize(&raw_transaction)?;

  let bitcoin_client = wallet.bitcoin_client();

  // process the PSBT.
  let psbt = bitcoin_client
    .wallet_process_psbt(
      &base64::engine::general_purpose::STANDARD
        .encode(Psbt::from_unsigned_tx(unsigned_transaction)?.serialize()),
      Some(true),
      None,
      Some(true),
    )?
    .psbt;

  // finalize & sign the PSBT.
  let finalized_psbt = bitcoin_client.finalize_psbt(&psbt, None)?;

  let signed_transaction = finalized_psbt
    .hex
    .clone()
    .ok_or_else(|| anyhow!("unable to sign transaction"))?;

  // broadcast the signed transaction.
  let transaction_id = bitcoin_client.send_raw_transaction(&signed_transaction)?;

  // write the bridge entry.
  wallet.track_bridge(
    transaction_id,
    BridgeEntry {
      script_key,
      batch_key,
      amount: amount as u64,
      rune_id: id,
      lock: true,
      psbt: finalized_psbt.psbt.unwrap(),
    },
  )?;

  Ok(Output {
    txid: transaction_id,
    asset_id,

    // TODO: use serde to hex encode (?).
    batch_key: hex::encode(batch_key),
    script_key: hex::encode(script_key),
  })
}

pub(crate) fn publish_proof(
  block: Block,
  transaction_index: u32,
  height: u32,
  entry: BridgeEntry,
  universe_url: String,
) -> Result<()> {
  let mut raw_block = Vec::new();
  block.consensus_encode(&mut raw_block)?;

  unsafe {
    TapPublishProof(&CProofConfig {
      packet: entry.psbt.as_ptr() as *const c_void,
      packet_length: entry.psbt.len() as c_int,
      block: raw_block.as_c_ptr() as *const c_void,
      block_length: raw_block.len() as c_int,
      transaction_index,
      height,
      amount: entry.amount,
      script_key: entry.script_key.as_c_ptr(),
      batch_key: entry.batch_key.as_c_ptr(),
      rune_id: &CRuneID {
        block: entry.rune_id.block,
        tx: entry.rune_id.tx,
      },
      universe_address: universe_url.as_ptr(),
    });
  }

  Ok(())
}
