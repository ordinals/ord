use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Traits {
  #[clap(help = "Show traits for <ORDINAL>.")]
  ordinal: Ordinal,
}

impl Traits {
  pub(crate) fn run(self) -> Result {
    print!("{}", self);
    Ok(())
  }
}

impl Display for Traits {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    writeln!(f, "number: {}", self.ordinal.n())?;
    writeln!(f, "decimal: {}", self.ordinal.decimal())?;
    writeln!(f, "degree: {}", self.ordinal.degree())?;
    writeln!(f, "name: {}", self.ordinal.name())?;
    writeln!(f, "height: {}", self.ordinal.height())?;
    writeln!(f, "cycle: {}", self.ordinal.cycle())?;
    writeln!(f, "epoch: {}", self.ordinal.epoch())?;
    writeln!(f, "period: {}", self.ordinal.period())?;
    writeln!(f, "offset: {}", self.ordinal.third())?;
    writeln!(f, "rarity: {}", self.ordinal.rarity())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn first() {
    assert_eq!(
      Traits {
        ordinal: Ordinal(0)
      }
      .to_string(),
      "\
number: 0
decimal: 0.0
degree: 0°0′0″0‴
name: nvtdijuwxlp
height: 0
cycle: 0
epoch: 0
period: 0
offset: 0
rarity: mythic
",
    );
  }

  #[test]
  fn last() {
    assert_eq!(
      Traits {
        ordinal: Ordinal(2099999997689999)
      }
      .to_string(),
      "\
number: 2099999997689999
decimal: 6929999.0
degree: 5°209999′1007″0‴
name: a
height: 6929999
cycle: 5
epoch: 32
period: 3437
offset: 0
rarity: uncommon
",
    );
  }
}
