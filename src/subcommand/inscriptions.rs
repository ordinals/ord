use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Inscriptions {
  #[clap(long, help = "Maximum number of inscriptions to list")]
  limit: Option<usize>,
  #[clap(long, help = "Maximum inscription number to list")]
  max_number: Option<u64>,
  #[clap(long, help = "Maximum inscription block height to list")]
  max_height: Option<u64>,
  #[clap(long, help = "Maximum sat number to list")]
  max_sat: Option<Sat>,
  #[clap(long, help = "Only list inscriptions on uncommon sats or rarer.")]
  uncommon: bool,
  #[clap(
    long,
    help = "List inscriptions in order of inscribed satoshi ordinals."
  )]
  order_by_sat: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct OutputWithSat {
  pub sat: Sat,
  pub number: u64,
  pub height: u64,
  pub timestamp: u32,
  pub inscription: InscriptionId,
  pub location: SatPoint,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct OutputWithoutSat {
  pub number: u64,
  pub height: u64,
  pub timestamp: u32,
  pub inscription: InscriptionId,
  pub location: SatPoint,
}

impl Inscriptions {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    let index_has_sats = index.has_sat_index()?;

    if !index_has_sats {
      if self.max_sat.is_some() {
        bail!("--max-sat requires index created with `--index-sats` flag")
      }

      if self.uncommon {
        bail!("--uncommon requires index created with `--index-sats` flag")
      }
    }

    let inscriptions = if self.order_by_sat {
      index.get_inscriptions_by_sat(
        self.limit,
        self.max_number,
        self.max_height,
        self.max_sat,
        self.uncommon,
      )?
    } else {
      index.get_inscriptions_by_inscription_number(
        self.limit,
        self.max_number,
        self.max_height,
        self.max_sat,
        self.uncommon,
      )?
    };

    let mut output_with_sat = Vec::new();
    let mut output_without_sat = Vec::new();

    for inscription in inscriptions {
      let entry = index
        .get_inscription_entry(inscription)?
        .ok_or_else(|| anyhow!("Inscription {inscription} not found"))?;
      if index_has_sats {
        output_with_sat.push(OutputWithSat {
          sat: entry.sat.unwrap(),
          inscription,
          location: index
            .get_inscription_satpoint_by_id(inscription)?
            .ok_or_else(|| anyhow!("Inscription {inscription} not found"))?,
          number: entry.number,
          height: entry.height,
          timestamp: entry.timestamp,
        });
      } else {
        output_without_sat.push(OutputWithoutSat {
          inscription,
          location: index
            .get_inscription_satpoint_by_id(inscription)?
            .ok_or_else(|| anyhow!("Inscription {inscription} not found"))?,
          number: entry.number,
          height: entry.height,
          timestamp: entry.timestamp,
        });
      }
    }

    if index_has_sats {
      print_json(&output_with_sat)?;
    } else {
      print_json(&output_without_sat)?;
    }

    Ok(())
  }
}
