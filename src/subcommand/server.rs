use super::*;

#[derive(Parser)]
pub(crate) struct Server {
  #[clap(long, default_value = "0.0.0.0")]
  address: String,
  #[clap(long, default_value = "80")]
  port: u16,
}

impl Server {
  pub(crate) fn run(self, options: Options) -> Result {
    Runtime::new()?.block_on(async {
      let index = Index::index(&options)?;

      let app = Router::new()
        .route("/list/:outpoint", get(Self::list))
        .route("/status", get(Self::status))
        .layer(extract::Extension(Arc::new(Mutex::new(index))));

      let addr = (self.address, self.port).to_socket_addrs()?.next().unwrap();

      let handle = Handle::new();

      LISTENERS.lock().unwrap().push(handle.clone());

      axum_server::Server::bind(addr)
        .handle(handle)
        .serve(app.into_make_service())
        .await?;

      Ok::<(), Error>(())
    })
  }

  async fn list(
    extract::Path(outpoint): extract::Path<OutPoint>,
    index: extract::Extension<Arc<Mutex<Index>>>,
  ) -> impl IntoResponse {
    match index.lock().unwrap().list(outpoint) {
      Ok(Some(ranges)) => (StatusCode::OK, Json(Some(ranges))),
      Ok(None) => (StatusCode::NOT_FOUND, Json(None)),
      Err(error) => {
        eprintln!("Error serving request for outpoint {outpoint}: {error}");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(None))
      }
    }
  }

  async fn status() -> StatusCode {
    StatusCode::OK
  }
}
