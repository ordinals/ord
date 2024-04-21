use super::*;

#[test]
fn airdrop() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  core.mine_blocks(1);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 1,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "100".parse().unwrap(),
        symbol: '¢',
        supply: "100".parse().unwrap(),
        turbo: false,
        ..default()
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  pretty_assert_eq!(
    CommandBuilder::new("--regtest wallet balance")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Balance>(),
    Balance {
      cardinal: 39999980000,
      ordinal: 10000,
      runic: Some(10000),
      runes: Some(
        vec![(
          SpacedRune {
            rune: Rune(RUNE),
            spacers: 0
          },
          "100".parse().unwrap()
        )]
        .into_iter()
        .collect()
      ),
      total: 400 * COIN_VALUE,
    }
  );


}
