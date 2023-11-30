use {
  colored::Colorize,
  reqwest::{blocking::get, StatusCode},
  std::process,
};

const ENDPOINTS: &[(&str, StatusCode, &str)] = &[
  (
    "/content/6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0",
    StatusCode::OK,
    "HIT",
  ),
  (
    "/content/6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0",
    StatusCode::OK,
    "HIT",
  ),
  ("/static/index.css", StatusCode::OK, "HIT"),
  ("/static/index.js", StatusCode::OK, "HIT"),
  ("/sat/FOO", StatusCode::BAD_REQUEST, "HIT"),
  ("/", StatusCode::OK, "BYPASS"),
  ("/blockheight", StatusCode::OK, "BYPASS"),
];

fn main() {
  eprint!("Warming up the cache");

  for (endpoint, expected_status_code, _expected_cache_status) in ENDPOINTS {
    let response = get(format!("https://ordinals.com{endpoint}")).unwrap();

    assert_eq!(response.status(), *expected_status_code);

    eprint!(".");
  }

  eprintln!();

  let mut failures = 0;

  for (endpoint, expected_status_code, expected_cache_status) in ENDPOINTS {
    eprint!("GET {endpoint}");

    let response = get(format!("https://ordinals.com{endpoint}")).unwrap();

    let status_code = response.status();

    eprint!(" {}", status_code.as_u16());

    assert_eq!(response.status(), *expected_status_code);

    let cache_status = response.headers().get("cf-cache-status").unwrap();

    let pass = cache_status == expected_cache_status;

    if pass {
      eprintln!(" {}", cache_status.to_str().unwrap().green());
    } else {
      eprintln!(" {}", cache_status.to_str().unwrap().red());
    }

    failures += u32::from(!pass);
  }

  if failures > 0 {
    eprintln!("{failures} failures");
    process::exit(1);
  }
}
