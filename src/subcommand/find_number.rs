use super::*;

#[derive(Debug, Parser)]
pub(crate) struct FindNumber {
  #[clap(help = "Find inscribe by number.")]
  number: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub inscription: InscriptionId,
}

impl FindNumber {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    // println!("request number: {:?}", self.number);
    let output = index.get_inscription_id_by_inscription_number(self.number);
    // println!("result: {:?}", output);

    let satpoint = index
      .get_inscription_satpoint_by_id(inscription_id)?
      .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

    
    if output.is_ok() {
        print_json(Output { inscription: output.unwrap().unwrap() } ).unwrap();
        Ok(())
    }else {
        Err(anyhow!("query inscribe by number failed"))
    }
  }
}
