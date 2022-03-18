use super::*;

pub(crate) fn run(options: Options) -> Result {
  let rt = Runtime::new()?;

  rt.block_on(async {
    let index = Index::index(&options)?;

    let app = Router::new()
      .route("/list/:outpoint", get(list))
      .layer(extract::Extension(Arc::new(Mutex::new(index))));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
      .serve(app.into_make_service())
      .await?;

    Ok::<(), Error>(())
  })?;

  Ok(())
}

async fn list(
  extract::Path(outpoint): extract::Path<OutPoint>,
  index: extract::Extension<Arc<Mutex<Index>>>,
) -> impl IntoResponse {
  match index.lock().unwrap().list(outpoint).unwrap() {
    Some(ranges) => (StatusCode::OK, Json(Some(ranges))),
    None => (StatusCode::NOT_FOUND, Json(None)),
  }
}
