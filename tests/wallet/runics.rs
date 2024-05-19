use {
  super::*,
  ord::{decimal::Decimal, subcommand::wallet::runics::RunicUtxo},
};

#[test]
fn wallet_runics() {
  let core = mockcore::builder().network(Network::Regtest).build();
  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        premine: "1000".parse().unwrap(),
        rune: SpacedRune { rune, spacers: 1 },
        supply: "1000".parse().unwrap(),
        symbol: '¢',
        terms: None,
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  pretty_assert_eq!(
    CommandBuilder::new("--regtest --index-runes wallet runics")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Vec<RunicUtxo>>()
      .first()
      .unwrap()
      .runes,
    vec![(
      SpacedRune { rune, spacers: 1 },
      Decimal {
        value: 1000,
        scale: 0
      }
    )]
    .into_iter()
    .collect()
  );
}
