use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Inscriptions {
  #[clap(long, help = "Maximum number of inscriptions to list")]
  max: Option<usize>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub sat: Sat,
  pub number: u64,
  pub inscription: InscriptionId,
  pub location: SatPoint,
}

impl Inscriptions {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    let inscriptions = index.get_inscriptions_by_sat(self.max)?;

    let mut output = Vec::new();

    for (sat, inscription) in inscriptions {
      output.push(Output {
        sat,
        inscription,
        location: index.get_inscription_satpoint_by_id(inscription)?.ok_or_else(|| anyhow!("Inscription {inscription} not found"))?,
        number: index.get_inscription_entry(inscription)?.ok_or_else(|| anyhow!("Inscription {inscription} not found"))?.number,
      });
    }

    print_json(&output)?;

    Ok(())
  }
}
