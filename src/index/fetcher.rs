use {
  crate::Options,
  anyhow::{anyhow, bail, ensure, Result},
  base64::Engine,
  bitcoin::{blockdata::block::Header, consensus::Decodable, Block, BlockHash, Transaction, Txid},
  // bitcoincore_rpc::Auth,
  hyper::{client::HttpConnector, Body, Client, Method, Request, Uri},
  serde::Deserialize,
  serde_json::{json, Value},
  std::{error::Error, fmt::Display},
};

#[derive(Clone)]
pub(crate) struct Fetcher {
  auth: String,
  client: Client<HttpConnector>,
  url: Uri,
}

#[derive(Deserialize, Debug)]
struct JsonResponse<T> {
  error: Option<JsonError>,
  id: usize,
  result: Option<T>,
}

#[derive(Deserialize, Debug)]
struct JsonError {
  code: i32,
  message: String,
}

impl Display for JsonError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "JSON-RPC error: code: {}, message: {}",
      self.code, self.message
    )
  }
}

impl Error for JsonError {}

const FETCH_SIZE: usize = 2000;

impl Fetcher {
  pub(crate) fn new(options: &Options) -> Result<Self> {
    let client = Client::new();

    let url = if options.rpc_url().starts_with("http://") {
      options.rpc_url()
    } else {
      "http://".to_string() + &options.rpc_url()
    };

    let url = Uri::try_from(&url).map_err(|e| anyhow!("Invalid rpc url {url}: {e}"))?;

    let (user, password) = options.auth()?.get_user_pass()?;
    let auth = format!("{}:{}", user.unwrap(), password.unwrap());
    let auth = format!(
      "Basic {}",
      &base64::engine::general_purpose::STANDARD.encode(auth)
    );
    Ok(Fetcher { client, url, auth })
  }

  pub(crate) async fn get_block_count(&self) -> Result<u64> {
    let count = self.fetch_json("getblockcount", json!([])).await?;
    Ok(count)
  }

  pub(crate) async fn get_block_hashes(&self, start_height: u64) -> Result<Vec<BlockHash>> {
    let range =
      usize::try_from(start_height).unwrap()..FETCH_SIZE + usize::try_from(start_height).unwrap();
    let res = self
      .batched_fetch_hex("getblockhash", &mut range.map(|i| json!([i])), true, true)
      .await?;
    Ok(res)
  }

  pub(crate) async fn get_block_headers(&self, hashes: &[BlockHash]) -> Result<Vec<Header>> {
    let res = self
      .batched_fetch_hex(
        "getblockheader",
        &mut hashes.iter().map(|hash| json!([hash, false])),
        false,
        false,
      )
      .await?;
    Ok(res)
  }

  pub(crate) async fn get_block(&self, hash: &BlockHash) -> Result<Block> {
    let block = self.fetch_hex("getblock", json!([hash, 0])).await?;
    Ok(block)
  }

  pub(crate) async fn get_transactions(&self, txids: Vec<Txid>) -> Result<Vec<Transaction>> {
    let txs = self
      .batched_fetch_hex(
        "getrawtransaction",
        &mut txids.iter().map(|txid| json!([txid])),
        false,
        false,
      )
      .await?;
    Ok(txs)
  }

  async fn fetch_json<T: for<'a> serde::de::Deserialize<'a>>(
    &self,
    method: &'static str,
    params: Value,
  ) -> Result<T> {
    let req = json!({
      "jsonrpc": "2.0",
      "id": 0,
      "method": method,
      "params": params
    });

    let body = req.to_string();
    let result: JsonResponse<T> = self.send_request(body).await?;

    ensure!(
      result.id == 0,
      "JSON-RPC response has different expected id {method}: {}",
      result.id
    );

    if let Some(err) = result.error {
      bail!(err);
    }

    let Some(result) = result.result else {
      bail!("JSON-RPC response had null result");
    };

    Ok(result)
  }

  async fn fetch_hex<T: Decodable>(&self, method: &'static str, params: Value) -> Result<T> {
    let str: String = self.fetch_json(method, params).await?;
    let hex = hex::decode(str)?;
    let res = bitcoin::consensus::deserialize(&hex)?;

    Ok(res)
  }

  async fn batched_fetch_hex<'a, T: Decodable>(
    &'a self,
    method: &'static str,
    params: &'a mut dyn Iterator<Item = Value>,
    allow_errors: bool,
    reverse_hex: bool,
  ) -> Result<Vec<T>> {
    let reqs = params
      .enumerate()
      .map(|(i, param)| {
        json!({
          "jsonrpc": "2.0",
          "id": i, // Use the index as id, so we can quickly sort the response
          "method": method,
          "params": param
        })
      })
      .collect::<Vec<_>>();

    if reqs.is_empty() {
      return Ok(Vec::new());
    }

    let body = Value::Array(reqs).to_string();
    let mut results: Vec<JsonResponse<String>> = self.send_request(body).await?;

    if !allow_errors {
      // Return early on any error, because we need all results to proceed
      if let Some(err) = results.iter().find_map(|res| res.error.as_ref()) {
        bail!("Error {err}");
      }
    } else {
      results.retain(|res| res.error.is_none());
    }

    // Results from batched JSON-RPC requests can come back in any order, so we must sort them by id
    results.sort_by(|a, b| a.id.cmp(&b.id));

    for (i, res) in results.iter().enumerate() {
      ensure!(i == res.id, "Missing response in batched JSON-RPC response");
    }

    let res = results
      .into_iter()
      .map(|res| {
        res
          .result
          .ok_or_else(|| anyhow!("Missing result for batched JSON-RPC response"))
          .and_then(|str| {
            hex::decode(str)
              .map_err(|e| anyhow!("Result for batched JSON-RPC response not valid hex: {e}"))
          })
          .and_then(|hex| {
            let hex = if reverse_hex {
              hex.into_iter().rev().collect()
            } else {
              hex
            };
            bitcoin::consensus::deserialize(&hex).map_err(|e| {
              anyhow!("Result for batched JSON-RPC response not valid bitcoin object: {e}")
            })
          })
      })
      .collect::<Result<Vec<T>>>()?;

    Ok(res)
  }

  async fn send_request<T: for<'a> serde::de::Deserialize<'a>>(&self, body: String) -> Result<T> {
    let req = Request::builder()
      .method(Method::POST)
      .uri(&self.url)
      .header(hyper::header::AUTHORIZATION, &self.auth)
      .header(hyper::header::CONTENT_TYPE, "application/json")
      .body(Body::from(body))?;
    let response = self.client.request(req).await?;
    let buf = hyper::body::to_bytes(response).await?;
    let res = serde_json::from_slice(&buf)?;
    Ok(res)
  }

  async fn try_get_transactions(&self, body: String) -> Result<Vec<JsonResponse<String>>> {
    let req = Request::builder()
      .method(Method::POST)
      .uri(&self.url)
      .header(hyper::header::AUTHORIZATION, &self.auth)
      .header(hyper::header::CONTENT_TYPE, "application/json")
      .body(Body::from(body))?;

    let response = self.client.request(req).await?;

    let buf = hyper::body::to_bytes(response).await?;

    let results: Vec<JsonResponse<String>> = match serde_json::from_slice(&buf) {
      Ok(results) => results,
      Err(e) => {
        return Err(anyhow!(
          "failed to parse JSON-RPC response: {e}. response: {response}",
          e = e,
          response = String::from_utf8_lossy(&buf)
        ))
      }
    };

    Ok(results)
  }
}
