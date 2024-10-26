use {super::*, ord::subcommand::wallet::addresses::Output};

#[test]
fn addresses() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);

  let etched = batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 3,
        premine: "1.111".parse().unwrap(),
        rune: SpacedRune { rune, spacers: 1 },
        supply: "2.222".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          amount: "1.111".parse().unwrap(),
          cap: 1,
          ..default()
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let output = CommandBuilder::new("--regtest --index-runes wallet addresses")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<BTreeMap<Address<NetworkUnchecked>, Vec<Output>>>();

  pretty_assert_eq!(
    output
      .get(&etched.output.rune.clone().unwrap().destination.unwrap())
      .unwrap(),
    &vec![Output {
      output: etched.output.rune.unwrap().location.unwrap(),
      amount: 10000,
      inscriptions: Some(Vec::new()),
      runes: Some(
        vec![(
          SpacedRune { rune, spacers: 1 },
          ord::decimal::Decimal {
            value: 1111,
            scale: 3,
          }
        )]
        .into_iter()
        .collect()
      ),
    }]
  );

  pretty_assert_eq!(
    output
      .get(&etched.output.inscriptions[0].destination)
      .unwrap(),
    &vec![Output {
      output: etched.output.inscriptions[0].location.outpoint,
      amount: 10000,
      inscriptions: Some(vec![etched.output.inscriptions[0].id]),
      runes: Some(BTreeMap::new()),
    }]
  );
}
