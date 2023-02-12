use {super::*, ord::subcommand::teleburn::Output};

#[test]
fn traits_command_prints_sat_traits() {
  assert_eq!(
    CommandBuilder::new(
      "teleburn 6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0"
    )
    .output::<Output>(),
    Output { ethereum: todo!() }
  );
}
