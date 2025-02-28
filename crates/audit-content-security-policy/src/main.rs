use {colored::Colorize, reqwest::blocking::get, std::process};

const SERVERS: &[(&str, &str, &str)] = &[
  (
    "signet.ordinals.net",
    "/content/7e1bc3b56b872aaf4d1aaf1565fac72182313c9142b207f9398afe263e234135i0",
    "https://signet.ordinals.com/content/",
  ),
  (
    "signet.ordinals.com",
    "/content/7e1bc3b56b872aaf4d1aaf1565fac72182313c9142b207f9398afe263e234135i0",
    "https://signet.ordinals.com/content/",
  ),
  (
    "alpha.ordinals.net",
    "/content/6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0",
    "https://ordinals.com/content/",
  ),
  (
    "bravo.ordinals.net",
    "/content/6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0",
    "https://ordinals.com/content/",
  ),
  (
    "charlie.ordinals.net",
    "/content/6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0",
    "https://ordinals.com/content/",
  ),
  (
    "ordinals.com",
    "/content/6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0",
    "https://ordinals.com/content/",
  ),
];

fn main() {
  let mut failures = 0;

  for (host, path, needle) in SERVERS {
    eprint!("GET {host}");

    let response = get(format!("https://{host}{path}")).unwrap();

    let mut fail = false;

    if !response.status().is_success() {
      eprint!(" {}", response.status().to_string().red());
      fail = true;
    }

    let headers = response.headers();

    let content_security_policy = headers
      .get("content-security-policy")
      .map(|value| value.to_str().unwrap().to_string())
      .unwrap_or_default();

    if !content_security_policy.contains(needle) {
      fail = true;
    }

    if fail {
      eprintln!(" {}", "FAIL".red());
      failures += 1;
    } else {
      eprintln!(" {}", "PASS".green());
    }
  }

  if failures > 0 {
    process::exit(1);
  }
}
