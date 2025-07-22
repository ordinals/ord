use {
  super::*,
  http_body_util::{BodyExt, Full},
  hyper::{body::Bytes, Method, Request, Uri},
  hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::TokioExecutor,
  },
  serde_json::{json, Value},
};

pub(crate) struct Fetcher {
  auth: String,
  client: Client<HttpConnector, Full<Bytes>>,
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

impl Fetcher {
  pub(crate) fn new(settings: &Settings) -> Result<Self> {
    let client = Client::builder(TokioExecutor::new()).build_http();

    let url = if settings.bitcoin_rpc_url(None).starts_with("http://") {
      settings.bitcoin_rpc_url(None)
    } else {
      "http://".to_string() + &settings.bitcoin_rpc_url(None)
    };

    let url = Uri::try_from(&url).map_err(|e| anyhow!("Invalid rpc url {url}: {e}"))?;

    let (user, password) = settings.bitcoin_credentials()?.get_user_pass()?;
    let auth = format!("{}:{}", user.unwrap(), password.unwrap());
    let auth = format!("Basic {}", &base64_encode(auth.as_bytes()));
    Ok(Fetcher { client, url, auth })
  }

  pub(crate) async fn get_transactions(&self, txids: Vec<Txid>) -> Result<Vec<Transaction>> {
    if txids.is_empty() {
      return Ok(Vec::new());
    }

    let mut reqs = Vec::with_capacity(txids.len());
    for (i, txid) in txids.iter().enumerate() {
      let req = json!({
        "jsonrpc": "2.0",
        "id": i, // Use the index as id, so we can quickly sort the response
        "method": "getrawtransaction",
        "params": [ txid ]
      });
      reqs.push(req);
    }

    let body = Value::Array(reqs).to_string();

    let mut results: Vec<JsonResponse<String>>;
    let mut retries = 0;

    loop {
      results = match self.try_get_transactions(body.clone()).await {
        Ok(results) => results,
        Err(error) => {
          if retries >= 5 {
            return Err(anyhow!(
              "failed to fetch raw transactions after 5 retries: {}",
              error
            ));
          }

          log::info!("failed to fetch raw transactions, retrying: {error}");

          tokio::time::sleep(Duration::from_millis(100 * u64::pow(2, retries))).await;
          retries += 1;
          continue;
        }
      };
      break;
    }

    // Return early on any error, because we need all results to proceed
    if let Some(err) = results.iter().find_map(|res| res.error.as_ref()) {
      return Err(anyhow!(
        "failed to fetch raw transaction: code {} message {}",
        err.code,
        err.message
      ));
    }

    // Results from batched JSON-RPC requests can come back in any order, so we must sort them by id
    results.sort_by(|a, b| a.id.cmp(&b.id));

    let txs = results
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
            consensus::deserialize(&hex).map_err(|e| {
              anyhow!("Result for batched JSON-RPC response not valid bitcoin tx: {e}")
            })
          })
      })
      .collect::<Result<Vec<Transaction>>>()?;
    Ok(txs)
  }

  async fn try_get_transactions(&self, body: String) -> Result<Vec<JsonResponse<String>>> {
    let req = Request::builder()
      .method(Method::POST)
      .uri(&self.url)
      .header(hyper::header::AUTHORIZATION, &self.auth)
      .header(hyper::header::CONTENT_TYPE, "application/json")
      .body(Full::new(Bytes::from(body)))?;

    let response = self.client.request(req).await?;

    let buf = response.into_body().collect().await?.to_bytes();

    let results: Vec<JsonResponse<String>> = match serde_json::from_slice(&buf) {
      Ok(results) => results,
      Err(e) => {
        return Err(anyhow!(
          "failed to parse JSON-RPC response: {e}. response: {response}",
          e = e,
          response = String::from_utf8_lossy(&buf)
        ));
      }
    };

    Ok(results)
  }
}
