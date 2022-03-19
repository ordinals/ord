use super::*;

pub(crate) fn run(options: Options) -> Result {
  Runtime::new()?.block_on(async {
    let index = Index::index(&options)?;

    let app = Router::new()
      .route("/list/:outpoint", get(list))
      .layer(extract::Extension(Arc::new(Mutex::new(index))));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

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
    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
  }
}
