use {
  anyhow::{anyhow, Result},
  bitcoin::{consensus::deserialize, Transaction, Txid},
  bitcoincore_rpc::Auth,
  hyper::{client::HttpConnector, Body, Client, Method, Request},
  serde::Deserialize,
};

pub(crate) struct TxFetcher {
  client: Client<HttpConnector>,
  url: String,
  auth: String,
}

#[derive(Deserialize, Debug)]
struct JsonResponse {
  result: Option<String>,
  error: Option<JsonError>,
  id: String,
}

#[derive(Deserialize, Debug)]
struct JsonError {
  code: i32,
  message: String,
}

impl TxFetcher {
  pub(crate) fn new(url: &str, auth: Auth) -> Result<Self> {
    if auth == Auth::None {
      return Err(anyhow!("No authentication provided"));
    }

    let client = Client::new();

    let url = if url.starts_with("http://") {
      url.to_string()
    } else {
      "http://".to_string() + url
    };

    let (user, password) = auth.get_user_pass()?;
    let auth = format!("{}:{}", user.unwrap(), password.unwrap());
    let auth = format!("Basic {}", &base64::encode(auth));
    Ok(TxFetcher { client, url, auth })
  }

  pub(crate) async fn get_transactions(&self, txids: Vec<Txid>) -> Result<Vec<Transaction>> {
    if txids.is_empty() {
      return Ok(Vec::new());
    }

    let mut reqs = Vec::with_capacity(txids.len());
    for (i, txid) in txids.iter().enumerate() {
      let req =
        format!("{{\"jsonrpc\":\"1.0\",\"id\":\"{i}\",\"method\":\"getrawtransaction\",\"params\":[\"{txid:x}\"]}}");
      reqs.push(req);
    }

    let body = format!("[{}]", reqs.join(","));
    let req = Request::builder()
      .method(Method::POST)
      .uri(&self.url)
      .header(hyper::header::AUTHORIZATION, &self.auth)
      .body(Body::from(body))?;

    let response = self.client.request(req).await?;

    let buf = hyper::body::to_bytes(response).await?;

    let mut results: Vec<JsonResponse> = serde_json::from_slice(&buf)?;

    if let Some(err) = results.iter().find_map(|res| res.error.as_ref()) {
      return Err(anyhow!(
        "Failed to fetch raw transaction: code {} message {}",
        err.code,
        err.message
      ));
    }

    results.sort_by(|a, b| {
      a.id
        .parse::<usize>()
        .unwrap()
        .cmp(&b.id.parse::<usize>().unwrap())
    });

    Ok(
      results
        .into_iter()
        .map(|res| deserialize(&hex::decode(res.result.unwrap()).unwrap()).unwrap())
        .collect(),
    )
  }
}
