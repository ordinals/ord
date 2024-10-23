#![allow(clippy::type_complexity)]

use {
  self::{command_builder::CommandBuilder, expected::Expected, test_server::TestServer},
  bitcoin::{
    address::{Address, NetworkUnchecked},
    Amount, Network, OutPoint, Sequence, Txid, Witness,
  },
  chrono::{DateTime, Utc},
  executable_path::executable_path,
  mockcore::TransactionTemplate,
  ord::{
    api, chain::Chain, outgoing::Outgoing, subcommand::runes::RuneInfo, wallet::batch,
    wallet::ListDescriptorsResult, InscriptionId, RuneEntry,
  },
  ordinals::{
    Artifact, Charm, Edict, Pile, Rarity, Rune, RuneId, Runestone, Sat, SatPoint, SpacedRune,
    COIN_VALUE,
  },
  pretty_assertions::assert_eq as pretty_assert_eq,
  regex::Regex,
  reqwest::{StatusCode, Url},
  serde::de::DeserializeOwned,
  std::sync::Arc,
  std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    str::{self, FromStr},
    thread,
    time::Duration,
  },
  tempfile::TempDir,
};

macro_rules! assert_regex_match {
  ($value:expr, $pattern:expr $(,)?) => {
    let regex = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
    let string = $value.to_string();

    if !regex.is_match(string.as_ref()) {
      eprintln!("Regex did not match:");
      pretty_assert_eq!(regex.as_str(), string);
    }
  };
}

mod command_builder;
mod expected;
mod test_server;

mod balances;
mod decode;
mod epochs;
mod find;
mod index;
mod info;
mod json_api;
mod list;
mod parse;
mod runes;
mod server;
mod settings;
mod subsidy;
mod supply;
mod traits;
mod verify;
mod version;
mod wallet;

const RUNE: u128 = 99246114928149462;

type Balance = ord::subcommand::wallet::balance::Output;
type Batch = ord::wallet::batch::Output;
type Create = ord::subcommand::wallet::create::Output;
type Inscriptions = Vec<ord::subcommand::wallet::inscriptions::Output>;
type Send = ord::subcommand::wallet::send::Output;
type Supply = ord::subcommand::supply::Output;

fn create_wallet(core: &mockcore::Handle, ord: &TestServer) {
  CommandBuilder::new(format!("--chain {} wallet create", core.network()))
    .core(core)
    .ord(ord)
    .stdout_regex(".*")
    .run_and_extract_stdout();
}

fn sats(
  core: &mockcore::Handle,
  ord: &TestServer,
) -> Vec<ord::subcommand::wallet::sats::OutputRare> {
  CommandBuilder::new(format!("--chain {} wallet sats", core.network()))
    .core(core)
    .ord(ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::sats::OutputRare>>()
}

fn inscribe_with_postage(
  core: &mockcore::Handle,
  ord: &TestServer,
  postage: Option<u64>,
) -> (InscriptionId, Txid) {
  core.mine_blocks(1);

  let mut command_str = format!(
    "--chain {} wallet inscribe --fee-rate 1 --file foo.txt",
    core.network()
  );

  if let Some(postage_value) = postage {
    command_str.push_str(&format!(" --postage {}sat", postage_value));
  }

  let output = CommandBuilder::new(command_str)
    .write("foo.txt", "FOO")
    .core(core)
    .ord(ord)
    .run_and_deserialize_output::<Batch>();

  core.mine_blocks(1);

  assert_eq!(output.inscriptions.len(), 1);

  (output.inscriptions[0].id, output.reveal)
}

fn inscribe(core: &mockcore::Handle, ord: &TestServer) -> (InscriptionId, Txid) {
  inscribe_with_postage(core, ord, None)
}

fn drain(core: &mockcore::Handle, ord: &TestServer) {
  let balance = CommandBuilder::new("--regtest --index-runes wallet balance")
    .core(core)
    .ord(ord)
    .run_and_deserialize_output::<Balance>();

  CommandBuilder::new(format!(
    "
      --chain regtest
      --index-runes
      wallet send
      --fee-rate 0
      bcrt1pyrmadgg78e38ewfv0an8c6eppk2fttv5vnuvz04yza60qau5va0saknu8k
      {}sat
    ",
    balance.cardinal
  ))
  .core(core)
  .ord(ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks_with_subsidy(1, 0);

  let balance = CommandBuilder::new("--regtest --index-runes wallet balance")
    .core(core)
    .ord(ord)
    .run_and_deserialize_output::<Balance>();

  pretty_assert_eq!(balance.cardinal, 0);
}

struct Etched {
  id: RuneId,
  output: Batch,
}

fn etch(core: &mockcore::Handle, ord: &TestServer, rune: Rune) -> Etched {
  batch(
    core,
    ord,
    batch::File {
      etching: Some(batch::Etching {
        supply: "1000".parse().unwrap(),
        divisibility: 0,
        terms: None,
        premine: "1000".parse().unwrap(),
        rune: SpacedRune { rune, spacers: 0 },
        symbol: '¢',
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  )
}

fn batch(core: &mockcore::Handle, ord: &TestServer, batchfile: batch::File) -> Etched {
  core.mine_blocks(1);

  let mut builder =
    CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
      .write("batch.yaml", serde_yaml::to_string(&batchfile).unwrap())
      .core(core)
      .ord(ord);

  for inscription in &batchfile.inscriptions {
    builder = builder.write(inscription.file.clone().unwrap(), "inscription");
  }

  let mut spawn = builder.spawn();

  let mut buffer = String::new();

  BufReader::new(spawn.child.stderr.as_mut().unwrap())
    .read_line(&mut buffer)
    .unwrap();

  assert_regex_match!(
    buffer,
    "Waiting for rune .* commitment [[:xdigit:]]{64} to mature…\n"
  );

  core.mine_blocks(5);

  let output = spawn.run_and_deserialize_output::<Batch>();

  core.mine_blocks(1);

  let block_height = core.height();

  let id = RuneId {
    block: block_height,
    tx: 1,
  };

  let reveal = output.reveal;
  let parent = output.inscriptions[0].id;

  let batch::Etching {
    divisibility,
    premine,
    rune,
    supply,
    symbol,
    terms,
    turbo,
  } = batchfile.etching.unwrap();

  {
    let supply = supply.to_integer(divisibility).unwrap();
    let premine = premine.to_integer(divisibility).unwrap();

    let mintable = terms
      .map(|terms| terms.cap * terms.amount.to_integer(divisibility).unwrap())
      .unwrap_or_default();

    assert_eq!(supply, premine + mintable);
  }

  let mut mint_definition = Vec::<String>::new();

  if let Some(terms) = terms {
    mint_definition.push("<dd>".into());
    mint_definition.push("<dl>".into());

    let mut mintable = true;

    mint_definition.push("<dt>start</dt>".into());
    {
      let relative = terms
        .offset
        .and_then(|range| range.start)
        .map(|start| start + block_height);
      let absolute = terms.height.and_then(|range| range.start);

      let start = relative
        .zip(absolute)
        .map(|(relative, absolute)| relative.max(absolute))
        .or(relative)
        .or(absolute);

      if let Some(start) = start {
        mintable &= block_height + 1 >= start;
        mint_definition.push(format!("<dd><a href=/block/{start}>{start}</a></dd>"));
      } else {
        mint_definition.push("<dd>none</dd>".into());
      }
    }

    mint_definition.push("<dt>end</dt>".into());
    {
      let relative = terms
        .offset
        .and_then(|range| range.end)
        .map(|end| end + block_height);
      let absolute = terms.height.and_then(|range| range.end);

      let end = relative
        .zip(absolute)
        .map(|(relative, absolute)| relative.min(absolute))
        .or(relative)
        .or(absolute);

      if let Some(end) = end {
        mintable &= block_height + 1 < end;
        mint_definition.push(format!("<dd><a href=/block/{end}>{end}</a></dd>"));
      } else {
        mint_definition.push("<dd>none</dd>".into());
      }
    }

    mint_definition.push("<dt>amount</dt>".into());

    mint_definition.push(format!(
      "<dd>{}</dd>",
      Pile {
        amount: terms.amount.to_integer(divisibility).unwrap(),
        divisibility,
        symbol: Some(symbol),
      }
    ));

    mint_definition.push("<dt>mints</dt>".into());
    mint_definition.push("<dd>0</dd>".into());
    mint_definition.push("<dt>cap</dt>".into());
    mint_definition.push(format!("<dd>{}</dd>", terms.cap));
    mint_definition.push("<dt>remaining</dt>".into());
    mint_definition.push(format!("<dd>{}</dd>", terms.cap));

    mint_definition.push("<dt>mintable</dt>".into());
    mint_definition.push(format!("<dd>{mintable}</dd>"));

    if mintable {
      mint_definition.push("<dt>progress</dt>".into());
      mint_definition.push("<dd>0%</dd>".into());
    }

    mint_definition.push("</dl>".into());
    mint_definition.push("</dd>".into());
  } else {
    mint_definition.push("<dd>no</dd>".into());
  }

  let RuneId { block, tx } = id;

  ord.assert_response_regex(
    format!("/rune/{rune}"),
    format!(
      r".*<dt>id</dt>
  <dd>{id}</dd>.*
  <dt>etching block</dt>
  <dd><a href=/block/{block}>{block}</a></dd>
  <dt>etching transaction</dt>
  <dd>{tx}</dd>
  <dt>mint</dt>
  {}
  <dt>supply</dt>
  <dd>{premine} {symbol}</dd>
  <dt>premine</dt>
  <dd>{premine} {symbol}</dd>
  <dt>premine percentage</dt>
  <dd>.*</dd>
  <dt>burned</dt>
  <dd>0 {symbol}</dd>
  <dt>divisibility</dt>
  <dd>{divisibility}</dd>
  <dt>symbol</dt>
  <dd>{symbol}</dd>
  <dt>turbo</dt>
  <dd>{turbo}</dd>
  <dt>etching</dt>
  <dd><a class=monospace href=/tx/{reveal}>{reveal}</a></dd>
  <dt>parent</dt>
  <dd><a class=monospace href=/inscription/{parent}>{parent}</a></dd>
.*",
      mint_definition.join("\\s+"),
    ),
  );

  let batch::RuneInfo {
    destination,
    location,
    rune: _,
  } = output.rune.clone().unwrap();

  if premine.to_integer(divisibility).unwrap() > 0 {
    let destination = destination
      .unwrap()
      .clone()
      .require_network(Network::Regtest)
      .unwrap();

    assert!(core.state().is_wallet_address(&destination));

    let location = location.unwrap();

    assert_eq!(core.address(location), destination);
  } else {
    assert!(destination.is_none());
    assert!(location.is_none());
  }

  let response = ord.json_request("/inscriptions");

  assert!(response.status().is_success());

  for id in response.json::<api::Inscriptions>().unwrap().ids {
    let response = ord.json_request(format!("/inscription/{id}"));
    assert!(response.status().is_success());
    if let Some(location) = location {
      let inscription = response.json::<api::Inscription>().unwrap();
      assert!(inscription.satpoint.outpoint != location);
    }
  }

  Etched { output, id }
}

fn envelope(payload: &[&[u8]]) -> Witness {
  let mut builder = bitcoin::script::Builder::new()
    .push_opcode(bitcoin::opcodes::OP_FALSE)
    .push_opcode(bitcoin::opcodes::all::OP_IF);

  for data in payload {
    let mut buf = bitcoin::script::PushBytesBuf::new();
    buf.extend_from_slice(data).unwrap();
    builder = builder.push_slice(buf);
  }

  let script = builder
    .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
    .into_script();

  Witness::from_slice(&[script.into_bytes(), Vec::new()])
}

fn default<T: Default>() -> T {
  Default::default()
}
