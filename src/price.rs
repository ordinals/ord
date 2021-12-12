use super::*;

pub fn get() -> Result<f64> {
  #[derive(Deserialize)]
  struct Response {
    bpi: Bpi,
  }

  #[derive(Deserialize)]
  struct Bpi {
    #[serde(rename = "USD")]
    usd: Usd,
  }

  #[derive(Deserialize)]
  struct Usd {
    rate_float: f64,
  }

  Ok(
    reqwest::blocking::get("https://api.coindesk.com/v1/bpi/currentprice/usd.json")?
      .json::<Response>()?
      .bpi
      .usd
      .rate_float,
  )
}
