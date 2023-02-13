use {
  anyhow::Result,
  bitcoin::{consensus::deserialize, BlockHash, Transaction, Txid},
  hyper::{client::HttpConnector, Client, Uri},
  std::str::FromStr,
};

pub(crate) struct Rest {
  client: Client<HttpConnector>,
  url: String,
}

impl Rest {
  pub(crate) fn new(url: &str) -> Self {
    let url = if !url.starts_with("http://") {
      "http://".to_string() + url
    } else {
      url.to_string()
    };
    Rest {
      client: Client::new(),
      url,
    }
  }

  pub(crate) async fn get_block_hash(&self, height: u32) -> Result<BlockHash> {
    let url = format!("{}/rest/blockhashbyheight/{height}.bin", self.url);
    let res = self.client.get(Uri::from_str(&url)?).await?;
    let buf = hyper::body::to_bytes(res).await?;
    let block_hash = deserialize(&buf)?;
    Ok(block_hash)
  }

  pub(crate) async fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction> {
    let url = format!("{}/rest/tx/{txid:x}.bin", self.url);
    let res = self.client.get(Uri::from_str(&url)?).await?;
    let buf = hyper::body::to_bytes(res).await?;
    let tx = deserialize(&buf)?;
    Ok(tx)
  }
}
