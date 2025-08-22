use crate::index::MEMPOOL_TRANSACTIONS;
use anyhow::Ok;
use bitcoin::{Txid, Weight};
use redb::{ReadableTable, WriteTransaction};
use serde::{Deserialize, Serialize};

use super::*;

pub(crate) struct MempoolIndexer {
  index: Arc<Index>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct MempoolTransaction {
  fee: u64,
  raw_tx: Vec<u8>,
  weight: Weight,
}

impl MempoolTransaction {
  pub(crate) fn store(&self) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&self.fee.to_le_bytes());
    buffer.extend_from_slice(&self.weight.to_wu().to_le_bytes());
    buffer.extend_from_slice(&self.raw_tx);
    buffer
  }
  #[allow(dead_code)]
  pub(crate) fn load(data: &[u8]) -> anyhow::Result<Self> {
    const U64_SIZE: usize = std::mem::size_of::<u64>();
    if data.len() < U64_SIZE * 2 {
      return Err(anyhow::anyhow!(
        "not enough data to decode MempoolTransaction"
      ));
    }
    let (fee_bytes, rest) = data.split_at(U64_SIZE);
    let fee = u64::from_le_bytes(fee_bytes.try_into()?);
    let (weight_bytes, rest) = rest.split_at(U64_SIZE);
    let weight = Weight::from_wu(u64::from_le_bytes(weight_bytes.try_into()?));

    let raw_tx = rest.to_vec();

    Ok(Self {
      fee,
      weight,
      raw_tx,
    })
  }
}

impl MempoolIndexer {
  pub fn new(index: Arc<Index>) -> Self {
    Self { index }
  }

  pub(crate) async fn run(self) {
    log::info!("Starting mempool indexer");
    loop {
      if let Err(err) = self.update_and_clean().await {
        log::error!("Mempool index error: {err}");
      }
      tokio::time::sleep(Duration::from_secs(3)).await
    }
  }

  async fn update_and_clean(&self) -> anyhow::Result<()>{
      let mut index_writer = self.index.begin_write()?;
      let wtx = &mut index_writer;

      self.update_mempool(wtx)?;
      self.clean_confirmed_transactions(wtx)?;

      index_writer.commit()?;

      Ok(())
  }

  pub(crate) fn update_mempool(&self, wtx: &mut WriteTransaction) -> anyhow::Result<()> {
    log::info!("Updating Mempool...");
    let mempool_txids: Vec<Txid> = self.index.client.get_raw_mempool()?;
    let mut table = wtx.open_table(MEMPOOL_TRANSACTIONS)?;

    for txid in mempool_txids {
      let key: &[u8] = txid.as_ref();

      if table.get(key)?.is_none() {
        log::debug!("New mempool transaction: {}", txid);
        match self.index.client.get_mempool_entry(&txid) {
          Result::Ok(entry) => {
            let raw_tx = self.index.client.get_raw_transaction(&txid, None)?;

            let mempool_tx = MempoolTransaction {
              fee: entry.fees.base.to_sat(),
              weight: Weight::from_wu(entry.weight.unwrap()),
              raw_tx: bitcoin::consensus::serialize(&raw_tx),
            };
            let serialize_tx = mempool_tx.store();
            table.insert(key, serialize_tx.as_slice())?;
          }
          Err(e) => {
            log::debug!("Could not fetch tx {}: {}", txid, e);
          }
        }
      }
    }

    Ok(())
  }

  pub(crate) fn clean_confirmed_transactions(
    &self,
    wtx: &mut WriteTransaction,
  ) -> anyhow::Result<()> {
    log::info!("Cleaning confirmed transactions from mempool index...");

    let mut table = wtx.open_table(MEMPOOL_TRANSACTIONS)?;
    let store_txids: Vec<[u8; 32]> = table
      .iter()?
      .map(|result| result.map(|(key, _value)| key.value().try_into().unwrap()))
      .collect::<Result<Vec<_>, _>>()?;

    let mut cleaned_count = 0;

    for txid_bytes in store_txids {
      let txid = Txid::from_byte_array(txid_bytes);
      if self.index.get_transaction_info(&txid)?.is_some() {
        log::debug!("Cleaning confirmed transaction: {}", txid);
        let key: &[u8] = txid.as_ref();
        table.remove(key)?;
        cleaned_count += 1;
      }
    }

    if cleaned_count > 0 {
      log::info!("Cleaned {} confirmed transactions.", cleaned_count);
    }

    Ok(())
  }
}
