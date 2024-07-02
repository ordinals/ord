use super::*;

#[test]
fn burning_rune_works() {
    let core = mockcore::builder().network(Network::Regtest).build();
  
    let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);
  
    create_wallet(&core, &ord);
  
    etch(&core, &ord, Rune(RUNE));
  
    let output = CommandBuilder::new(format!(
      "--chain regtest --index-runes wallet send --fee-rate 1 bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 100:{}",
      Rune(RUNE)
    ))
    .core(&core)
      .ord(&ord)
    .run_and_deserialize_output::<Send>();
  
    core.mine_blocks(1);
  
    let balances = CommandBuilder::new("--regtest --index-runes balances")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<ord::subcommand::balances::Output>();
  
    assert_eq!(
      balances,
      ord::subcommand::balances::Output {
        runes: vec![(
          SpacedRune::new(Rune(RUNE), 0),
          vec![(
            OutPoint {
                txid: output.txid,
                vout: 1
            },
            Pile {
                amount: 900,
                divisibility: 0,
                symbol: Some('¢')
            },
          ),(
            OutPoint {
              txid: output.txid,
              vout: 2
            },
            Pile {
              amount: 100,
              divisibility: 0,
              symbol: Some('¢')
            },
          )]
          .into_iter()
          .collect()
        ),]
        .into_iter()
        .collect(),
      }
    );

    // burn all runes left in wallet
    // defining amount has no use
    let _ = CommandBuilder::new(format!("--chain regtest --index-runes wallet burn --fee-rate 1 500:{}", 
        Rune(RUNE)))
    .core(&core)
      .ord(&ord)
    .run_and_deserialize_output::<Burn>();

    core.mine_blocks(1);

    // burned amount should not be picked up by updater
    let balances = CommandBuilder::new("--regtest --index-runes balances")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<ord::subcommand::balances::Output>();

    pretty_assert_eq!(
      balances,
      ord::subcommand::balances::Output {
        runes: vec![(
          SpacedRune::new(Rune(RUNE), 0),
          vec![(
            OutPoint {
              txid: output.txid,
              vout: 2
            },
            Pile {
              amount: 100,
              divisibility: 0,
              symbol: Some('¢')
            },
          )]
          .into_iter()
          .collect()
        ),]
        .into_iter()
        .collect(),
      }
    );

    let rune = CommandBuilder::new("--regtest --index-runes runes")
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<ord::subcommand::runes::Output>();

    let burned = rune.runes[&Rune(RUNE)].burned;

    assert_eq!(
      burned,
      900
    );
  }