use std::fmt;
use std::io;
use std::time::Duration;

use bitcoincore_rpc::jsonrpc;
use bitcoincore_rpc::jsonrpc::{Request, Response, Transport};
use http::Method;
use hyper::client;
use hyper::client::HttpConnector;
use hyper_rustls::{ConfigBuilderExt, HttpsConnector};

pub struct CustomTransport {
    url: String,
    timeout: Duration,
    client: hyper::Client<HttpsConnector<HttpConnector>>,
    is_https: bool,
    auth: String,
}

fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}
impl CustomTransport {
    /// Construct a new `SimpleHttpTransport` with default parameters
    pub fn new(url: &str, auth: &str) -> Self {
        // Prepare the TLS client config
        let tls = rustls::ClientConfig::builder()
            .with_native_roots().unwrap_or_else(|err| panic!("init tls failed: {}", err))
            .with_no_client_auth();
        // Prepare the HTTPS connector
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(tls)
            .https_or_http()
            .enable_http1()
            .build();
        let client: client::Client<_, hyper::Body> = client::Client::builder().build(https);

        CustomTransport {
            url: url.to_string(),
            timeout: Duration::from_secs(15),
            client: client,
            is_https: url.starts_with("https://"),
            auth: auth.to_string(),
        }
    }


    fn request<R>(&self, req: impl serde::Serialize) -> std::result::Result<R, jsonrpc::simple_http::Error>
        where
            R: for<'a> serde::de::Deserialize<'a>,
    {
        let payload = serde_json::to_string(&req).unwrap();
        let req_body = http::Request::builder()
            .method(Method::POST)
            .uri(&self.url)
            .header(hyper::header::AUTHORIZATION, &self.auth)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .body(hyper::Body::from(payload)).expect("cannot send request");

        let httpclient= self.client.clone();
        let response =  std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                let response= httpclient.request(req_body).await.expect("cannot send request");
                let body_bytes = hyper::body::to_bytes(response.into_body()).await.expect("no response");
                body_bytes
            })
        }).join();

        let result = serde_json::from_slice(&response.unwrap()).unwrap();
        Ok(result)
    }
}

impl Transport for CustomTransport {
    fn send_request(&self, req: Request) -> Result<Response, jsonrpc::Error> {
        Ok(self.request(req)?)
    }

    fn send_batch(&self, reqs: &[Request]) -> Result<Vec<Response>, jsonrpc::Error> {
        Ok(self.request(reqs)?)
    }

    fn fmt_target(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}