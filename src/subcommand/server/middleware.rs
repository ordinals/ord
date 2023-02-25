use axum::{extract::MatchedPath, http::Request, middleware::Next, response::Response};
use opentelemetry::{
  global,
  trace::{Span, Tracer},
  Key,
};

pub(crate) async fn tracing_layer<B>(request: Request<B>, next: Next<B>) -> Response {
  let tracer = global::tracer("ord-kafka");
  let cx = opentelemetry::Context::current();
  let route = request
    .extensions()
    .get::<MatchedPath>()
    .unwrap()
    .as_str()
    .to_string();
  let uri = request.uri();
  let host = uri.host().unwrap_or_default();
  let path = uri.path();
  let path_and_query = uri
    .path_and_query()
    .map(|x| x.as_str())
    .unwrap_or_else(|| "/");

  let mut span = tracer.start_with_context(route.clone(), &cx);
  span.set_attribute(Key::new("http.method").string(request.method().as_str().to_string()));
  span.set_attribute(Key::new("http.target").string(path.to_string()));
  span.set_attribute(Key::new("http.host").string(host.to_string()));
  span.set_attribute(Key::new("http.route").string(route));
  span.set_attribute(Key::new("http.url").string(format!(
    "{}://{}{}",
    uri.scheme_str().unwrap_or_default(),
    host,
    path_and_query
  )));

  let useragent = request
    .headers()
    .get("user-agent")
    .map(|x| x.to_str().unwrap_or_default());

  if let Some(useragent) = useragent {
    span.set_attribute(Key::new("http.useragent").string(useragent.to_string()));
  }

  let response = next.run(request).await;

  // Set http response
  span.set_attribute(Key::new("http.status_code").i64(i64::from(response.status().as_u16())));
  span.end();

  response
}
