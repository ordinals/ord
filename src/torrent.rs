use {
  super::{inscription::Inscription, Error},
  anyhow::Context,
  bitcoin::hashes::{sha256, Hash},
  lava_torrent::{bencode::BencodeElem, torrent::v1::TorrentBuilder},
  std::path::{Path, PathBuf},
  std::{ffi::OsString, fs},
  urlencoding::encode,
};

use bitcoin::hashes::hex::ToHex;

// Content type to identify Bittorent v1 hash commitments (v1 is implied by btih, v2 uses btmh)
const CONTENT_TYPE: &str = "sha2,btih";

// Bittorrent piece length (1MB)
const PIECE_LENGTH: i64 = 1048576;

// Default tracker URIs (included in .torrent & magnet links)
// TODO support multiple, add wss tracker
pub const DEFAULT_TRACKER: &str = "udp://tracker.ordinals.com";

// Default webseed URL (included in .torrent & magnet links)
// Expected to serve files at WEBSEED_URL/INFOHASH/FILENAME and the .torrent metadata file at WEBSEED_URL/INFOHASH.torrent
pub const DEFAULT_WEBSEED: &str = "https://webseed.ordinals.com";

// Bootstrap peers for DHT discovery (included in .torrent & magnet links)
pub const DEFAULT_PEER: &str = "dht.ordinals.com:6885";

pub(crate) fn make_torrent_inscription(
  file_path: impl AsRef<Path>,
  torrent_path: Option<impl AsRef<Path>>,
  tracker_url: &str,
  peer_addr: &str,
) -> Result<Inscription, Error> {
  // TorrentBuilder requires absolute paths
  let file_path = fs::canonicalize(file_path)?;

  // Create the torrent and gets its infohash
  let torrent = TorrentBuilder::new(&file_path, PIECE_LENGTH)
    .set_announce(Some(tracker_url.to_string()))
    .add_extra_field("nodes".to_string(), bencode_nodes(peer_addr))
    .build()
    .with_context(|| "TorrentBuilder failed")?;
  let infohash = torrent.info_hash_bytes();

  // Write the .torrent file (by default, to <path>.torrent)
  let torrent_path = get_torrent_path(&file_path, torrent_path);
  log::info!(
    "Writing torrent with infohash {} to {}",
    infohash.to_hex(),
    torrent_path.display()
  );
  torrent
    .write_into_file(torrent_path)
    .with_context(|| "failed writing .torrent file")?;

  // Calculate the file's SHA256
  // TODO streaming hash to avoid loading the entire file in memory
  let contents =
    fs::read(&file_path).with_context(|| format!("io error reading {}", file_path.display()))?;
  let sha256hash = sha256::Hash::hash(&contents).into_inner().to_vec();

  // Create an inscription for the SHA256+infohash
  Ok(Inscription::new(
    Some(CONTENT_TYPE.into()),
    Some([sha256hash, infohash].concat()),
  ))
}

fn get_torrent_path(file_path: &Path, torrent_path: Option<impl AsRef<Path>>) -> PathBuf {
  if let Some(torrent_path) = torrent_path {
    torrent_path.as_ref().to_owned()
  } else {
    let mut fileoss: OsString = file_path.into();
    fileoss.push(".torrent");
    fileoss.into()
  }
}

fn bencode_nodes(nodes: &str) -> BencodeElem {
  BencodeElem::List(
    nodes
      .trim()
      .split(" ")
      .filter_map(|node| {
        let mut parts = node.split(":"); // host:port
        Some(BencodeElem::List(vec![
          parts.next()?.into(),
          parts.next()?.parse::<i64>().ok()?.into(),
        ]))
      })
      .collect(),
  )
}
