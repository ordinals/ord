use {super::*, ord::sat::Output, ord::Rarity};

#[test]
fn traits_command_prints_sat_traits() {
  assert_eq!(
    CommandBuilder::new("traits 0").run_and_check_output::<Output>(),
    Output {
      number: 0,
      decimal: "0.0".into(),
      degree: "0°0′0″0‴".into(),
      name: "nvtdijuwxlp".into(),
      block: 0,
      cycle: 0,
      epoch: 0,
      period: 0,
      offset: 0,
      rarity: Rarity::Mythic,
      percentile: "0%".into(),
    }
  );
}
#[test]
fn traits_command_for_last_sat() {
  assert_eq!(
    CommandBuilder::new("traits 2099999997689999").run_and_check_output::<Output>(),
    Output {
      number: 2099999997689999,
      decimal: "6929999.0".into(),
      degree: "5°209999′1007″0‴".into(),
      name: "a".into(),
      block: 6929999,
      cycle: 5,
      epoch: 32,
      period: 3437,
      offset: 0,
      rarity: Rarity::Uncommon,
      percentile: "100%".into(),
    }
  );
}
