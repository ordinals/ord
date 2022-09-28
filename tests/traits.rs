use super::*;

#[test]
fn traits_command_prints_ordinal_traits() {
  CommandBuilder::new("traits 0")
    .expected_stdout(
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
    )
    .run();
}
