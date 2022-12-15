use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Traits {
  #[clap(help = "Show traits for <SAT>.")]
  sat: Sat,
}

impl Traits {
  pub(crate) fn run(self) -> Result {
    print!("{}", self);
    Ok(())
  }
}

impl Display for Traits {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    writeln!(f, "number: {}", self.sat.n())?;
    writeln!(f, "decimal: {}", self.sat.decimal())?;
    writeln!(f, "degree: {}", self.sat.degree())?;
    writeln!(f, "name: {}", self.sat.name())?;
    writeln!(f, "height: {}", self.sat.height())?;
    writeln!(f, "cycle: {}", self.sat.cycle())?;
    writeln!(f, "epoch: {}", self.sat.epoch())?;
    writeln!(f, "period: {}", self.sat.period())?;
    writeln!(f, "offset: {}", self.sat.third())?;
    writeln!(f, "rarity: {}", self.sat.rarity())?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn first() {
    assert_eq!(
      Traits { sat: Sat(0) }.to_string(),
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
        sat: Sat(2099999997689999)
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
