use {
  crate::Network,
  anyhow::{bail, ensure, Context, Result},
  bitcoin::{
    blockdata::block::{Block, BlockHash, Header},
    consensus::{encode, Decodable},
    hashes::Hash,
    network::{
      constants,
      message::{NetworkMessage, RawNetworkMessage},
      message_blockdata::{GetHeadersMessage, Inventory},
      message_network::VersionMessage,
      Address,
    },
    secp256k1::{self, rand::Rng},
  },
  log::{debug, trace, warn},
  std::{
    io::{ErrorKind, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    sync::{
      mpsc::{sync_channel, Receiver, SyncSender},
      Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
  },
};

pub(crate) struct Connection {
  msg_send: SyncSender<NetworkMessage>,
  blocks_recv: Receiver<Block>,
  headers_recv: Receiver<Vec<Header>>,
}

impl Connection {
  pub(crate) fn get_headers(&mut self, begin_hash: Option<BlockHash>) -> Result<Vec<Header>> {
    let msg = GetHeadersMessage::new(
      vec![begin_hash.unwrap_or_else(BlockHash::all_zeros)],
      BlockHash::all_zeros(),
    );

    self.msg_send.send(NetworkMessage::GetHeaders(msg))?;

    let headers = self.headers_recv.recv()?;

    Ok(headers)
  }

  pub(crate) fn for_blocks<F>(&mut self, hashes: &[BlockHash], mut func: F) -> Result<()>
  where
    F: FnMut(Block),
  {
    let invs: Vec<Inventory> = hashes
      .iter()
      .map(|hash| Inventory::WitnessBlock(*hash))
      .collect();

    self.msg_send.send(NetworkMessage::GetData(invs))?;

    for hash in hashes {
      let block = self.blocks_recv.recv()?;
      ensure!(
        block.block_hash() == *hash,
        "got unexpected block for {hash:?}"
      );
      func(block);
    }

    Ok(())
  }

  pub(crate) fn connect(network: Network, address: SocketAddr) -> Result<Self> {
    let conn = Arc::new(
      TcpStream::connect(address)
        .with_context(|| format!("Failed to connect to {network} node at {address:?}"))?,
    );

    let (tx_send, tx_recv) = sync_channel::<NetworkMessage>(1);

    let stream = Arc::clone(&conn);
    crate::thread::spawn(move || loop {
      use std::net::Shutdown;
      let msg = match tx_recv.recv() {
        Ok(msg) => msg,
        Err(_) => {
          // p2p_loop is closed, so tx_send is disconnected
          debug!("closing p2p_send thread: no more messages to send");
          // close the stream reader (p2p_recv thread may block on it)
          if let Err(e) = stream.shutdown(Shutdown::Read) {
            warn!("failed to shutdown p2p connection: {}", e)
          }
          return;
        }
      };
      trace!("send: {:?}", msg);
      let raw_msg = RawNetworkMessage {
        magic: network.magic(),
        payload: msg,
      };
      let _ = (&*stream)
        .write_all(encode::serialize(&raw_msg).as_slice())
        .context("p2p failed to send");
    });

    let (blocks_send, blocks_recv) = sync_channel::<Block>(10);
    let (headers_send, headers_recv) = sync_channel::<Vec<Header>>(1);
    let (init_send, init_recv) = sync_channel::<()>(0);

    tx_send.send(build_version_message())?;

    let stream = Arc::clone(&conn);
    let tx_send_clone = tx_send.clone();
    crate::thread::spawn(move || loop {
      let raw_msg = RawNetworkMessage::consensus_decode(&mut &*stream);

      let msg = match raw_msg {
        Ok(raw_msg) => {
          assert_eq!(raw_msg.magic, network.magic());
          raw_msg.payload
        }
        Err(encode::Error::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => {
          debug!("closing p2p_recv thread: connection closed");
          return Ok(());
        }
        Err(e) => bail!("failed to recv a message from peer: {}", e),
      };

      match msg {
        NetworkMessage::GetHeaders(_) => {
          tx_send_clone.send(NetworkMessage::Headers(vec![]))?;
        }
        NetworkMessage::Version(version) => {
          debug!("peer version: {:?}", version);
          tx_send_clone.send(NetworkMessage::Verack)?;
        }
        NetworkMessage::Inv(_) => (),
        NetworkMessage::Ping(nonce) => {
          tx_send_clone.send(NetworkMessage::Pong(nonce))?; // connection keep-alive
        }
        NetworkMessage::Verack => {
          init_send.send(())?; // peer acknowledged our version
        }
        NetworkMessage::Block(block) => blocks_send.send(block)?,
        NetworkMessage::Headers(headers) => headers_send.send(headers)?,
        NetworkMessage::Alert(_) => (), // https://bitcoin.org/en/alert/2016-11-01-alert-retirement
        NetworkMessage::Addr(_) => (),  // unused
        msg => warn!("unexpected message: {:?}", msg),
      }
    });

    init_recv.recv()?; // wait until `verack` is received

    Ok(Connection {
      msg_send: tx_send,
      blocks_recv,
      headers_recv,
    })
  }
}

fn build_version_message() -> NetworkMessage {
  let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
  let timestamp: i64 = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time error")
    .as_secs()
    .try_into()
    .unwrap();

  let services = constants::ServiceFlags::NONE;
  const VERSION: &str = env!("CARGO_PKG_VERSION");

  NetworkMessage::Version(VersionMessage {
    version: constants::PROTOCOL_VERSION,
    services,
    timestamp,
    receiver: Address::new(&addr, services),
    sender: Address::new(&addr, services),
    nonce: secp256k1::rand::thread_rng().gen(),
    user_agent: format!("/ord:{VERSION}/"),
    start_height: 0,
    relay: false,
  })
}
