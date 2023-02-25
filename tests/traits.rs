use {super::*, ord::subcommand::traits::Output, ord::Rarity};

#[test]
fn traits_command_prints_sat_traits() {
  assert_eq!(
    CommandBuilder::new("traits 0").output::<Output>(),
    Output {
      number: 0,
      decimal: "0.0".into(),
      degree: "0°0′0″0‴".into(),
      name: "bgmbqkqiqsxl".into(),
      height: 0,
      cycle: 0,
      epoch: 0,
      period: 0,
      offset: 0,
      rarity: Rarity::Mythic,
    }
  );
}
#[test]
fn traits_command_for_last_sat() {
  assert_eq!(
    CommandBuilder::new("traits 8399999990759999").output::<Output>(),
    Output {
      number: 8399999990759999,
      decimal: "27719999.0".into(),
      degree: "10°839999′2015″0‴".into(),
      name: "a".into(),
      height: 27719999,
      cycle: 10,
      epoch: 32,
      period: 13749,
      offset: 0,
      rarity: Rarity::Uncommon,
    }
  );
}
