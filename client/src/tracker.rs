use crate::error::Error;
use crate::model::InfoDict;
use crate::peer::Peer;
use async_trait::async_trait;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use log::{debug, error, warn};
use rand::Rng;
use sha1::Digest;
use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::time::Duration;
use tokio::io;
use tokio::net::UdpSocket;
use tokio::time::timeout;

const MAGIC_CONSTANT: i64 = 0x41727101980;
const RECV_BUF_SIZE: usize = 1024;

const ACTION_CONNECT: i32 = 0;
const ACTION_ANNOUNCE: i32 = 1;
const ACTION_SCRAPE: i32 = 2;
const ACTION_ERROR: i32 = 3;

// const EVENT_NONE: i32 = 0;
// const EVENT_COMPLETED: i32 = 1;
const EVENT_STARTED: i32 = 2;
// const EVENT_STOPPED: i32 = 3;

#[async_trait]
pub trait TrackerClient {
    /// Allows the user to announce its existence to the tracker that this client represents.
    async fn announce(&mut self, info_dict: &InfoDict) -> Result<AnnounceResponsePayload, Error>;
}

#[derive(Debug)]
pub struct Connection {
    addr: SocketAddr,
    socket: UdpSocket,
    id: i64,
    port: u16,
}

#[derive(Debug)]
struct ConnectResponsePayload {
    transaction_id: i32,
    action: i32,
    connection_id: i64,
}

#[derive(Debug)]
enum ConnectResponse {
    Payload(ConnectResponsePayload),
    Error(String),
}

#[derive(Debug)]
pub struct AnnounceResponsePayload {
    transaction_id: i32,
    interval: std::time::Duration,
    num_leechers: i32,
    num_seeders: i32,
    pub peers: Vec<Peer>,
}

#[derive(Debug)]
enum AnnounceResponse {
    Payload(AnnounceResponsePayload),
    Error(String),
}

impl Connection {
    pub async fn new(addr: SocketAddr) -> Result<Connection, Error> {
        if let Some((mut socket, port)) = try_bind_socket().await {
            socket.connect(&addr).await.map_err(Error::Tokio)?;
            let connection_id = connect(&mut socket).await?;

            debug!(
                "socket connected to addr {} with id {}",
                addr, connection_id
            );

            Ok(Connection {
                addr: addr,
                socket: socket,
                id: connection_id,
                port: port,
            })
        } else {
            Err(Error::PortsExhausted)
        }
    }

    pub async fn scrape(&mut self) -> Result<(), Error> {
        let transaction_id = get_transaction_id();
        let scrape_req = get_scrape_request(self.id, transaction_id);

        self.socket.send(&scrape_req).await?;
        let mut buf = [0u8; RECV_BUF_SIZE];

        let len = timeout(Duration::from_secs(2), self.socket.recv(&mut buf))
            .await
            .map_err(|e| {
                error!("attempt to receive announce response timed out: {}", e);
                Error::Timeout
            })??;

        debug!("[scrape] read {} bytes from dgram", len);
        assert!(len >= 20);

        Ok(())
    }

    pub async fn announce(
        &mut self,
        info_dict: &InfoDict,
    ) -> Result<AnnounceResponsePayload, Error> {
        let hashed_info_dict = get_hashed_info_dict(info_dict);
        let transaction_id = get_transaction_id();
        let peer_id = get_peer_id();
        let announce_req = get_announce_request(
            self.id,
            transaction_id,
            self.port,
            &hashed_info_dict,
            &peer_id,
        );

        self.socket.send(&announce_req).await?;
        let mut buf = [0u8; RECV_BUF_SIZE];

        let len = timeout(Duration::from_secs(2), self.socket.recv(&mut buf))
            .await
            .map_err(|e| {
                error!("attempt to receive announce response timed out: {}", e);
                Error::Timeout
            })??;

        debug!("read {} bytes from dgram", len);
        assert!(len >= 20);

        // TODO: add exponential backoff
        let announce_res = read_announce_response(&buf, len)
            .map_err(|e| Error::Tokio(io::Error::new(io::ErrorKind::InvalidData, e)))?;

        match announce_res {
            AnnounceResponse::Payload(res) => {
                if res.transaction_id != transaction_id {
                    return Err(Error::Server(
                        "received incorrect transaction id".to_owned(),
                    ));
                }
                debug!("leechers: {}", res.num_leechers);
                debug!("seeders: {}", res.num_seeders);
                debug!("interval: {:?}", res.interval);
                debug!("peers (len = {})", res.peers.len());
                for peer in res.peers.iter() {
                    debug!("    {} => {}", peer.ip, peer);
                }

                Ok(res)
            }
            AnnounceResponse::Error(s) => Err(Error::Server(s)),
        }
    }
}

fn get_hashed_info_dict(info_dict: &InfoDict) -> Vec<u8> {
    let info_dict_bytes =
        bencoding::to_bytes(info_dict).expect("info dict should not fail to encode");
    {
        use std::io::Write;
        let mut f = std::fs::File::create("rust_myfile").unwrap();
        f.write_all(&info_dict_bytes).unwrap();
    }

    let mut hasher = sha1::Sha1::new();
    hasher.update(info_dict_bytes);

    let bytes = &hasher.finalize();
    assert!(bytes.len() == 20);

    debug!("info_hash: {:02x}", &bytes);

    bytes.to_vec()
}

async fn try_bind_socket() -> Option<(UdpSocket, u16)> {
    let port_range: RangeInclusive<u16> = 6881..=6889;
    for port in port_range {
        let local_addr: SocketAddr = format!("0.0.0.0:{}", port).parse().ok()?;
        let socket = match UdpSocket::bind(&local_addr).await {
            Ok(s) => s,
            Err(e) => {
                warn!("failed to bind to socket on port {}: {}", port, e);
                continue;
            }
        };
        return Some((socket, port));
    }
    None
}

async fn connect(socket: &mut UdpSocket) -> Result<i64, Error> {
    let transaction_id = get_transaction_id();
    let connect_req = get_connect_request(transaction_id);

    debug!(
        "connecting to tracker with transaction_id {}",
        transaction_id
    );
    socket.send(&connect_req).await.map_err(Error::Tokio)?;

    let mut buf = [0u8; RECV_BUF_SIZE];

    let len = timeout(Duration::from_secs(2), socket.recv(&mut buf))
        .await
        .map_err(|e| {
            error!("attempt to connect timed out: {}", e);
            Error::Timeout
        })??;

    assert!(len >= 16);

    match read_connect_response(&buf, len)? {
        ConnectResponse::Payload(res) => {
            if res.transaction_id != transaction_id {
                error!("received incorrect transaction id");
                return Err(Error::IncorrectTransactionId);
            }

            Ok(res.connection_id)
        }
        ConnectResponse::Error(s) => Err(Error::Server(s)),
    }
}

fn get_peer_id() -> Vec<u8> {
    const PEER_ID_SIZE: usize = 20;

    let mut res = vec![];
    std::io::Write::write(&mut res, b"TH-0.1.0---").unwrap();

    let bytes_remaining = PEER_ID_SIZE - res.len();
    for _ in 0..bytes_remaining {
        // generate a random ascii byte
        let byte = rand::thread_rng().gen::<u8>();
        res.write_u8(byte).unwrap();
    }

    // the peer id should have 20 bytes in size
    assert!(res.len() == PEER_ID_SIZE);
    res
}

fn get_transaction_id() -> i32 {
    // A transaction id is just a random i32
    rand::thread_rng().gen::<i32>()
}

fn get_random_key() -> u32 {
    rand::thread_rng().gen::<u32>()
}

fn get_connect_request(transaction_id: i32) -> Vec<u8> {
    // protocol id
    let mut writer = vec![];
    writer.write_i64::<BigEndian>(MAGIC_CONSTANT).unwrap();
    writer.write_i32::<BigEndian>(ACTION_CONNECT).unwrap();
    writer.write_i32::<BigEndian>(transaction_id).unwrap();
    writer
}

fn read_connect_response(buf: &[u8], nread: usize) -> Result<ConnectResponse, Error> {
    assert!(nread > 4);
    let mut reader = std::io::Cursor::new(buf);
    let action = reader.read_i32::<BigEndian>()?;
    let recv_transaction_id = reader.read_i32::<BigEndian>()?;

    if action == ACTION_CONNECT {
        assert!(nread == 16);
        let connection_id = reader.read_i64::<BigEndian>()?;
        Ok(ConnectResponse::Payload(ConnectResponsePayload {
            action: action,
            connection_id: connection_id,
            transaction_id: recv_transaction_id,
        }))
    } else if action == ACTION_ERROR {
        assert!(nread >= 8);
        let mut error_vec = vec![0u8; nread - 8];
        std::io::Read::read_exact(&mut reader, &mut error_vec).unwrap();
        let error_string = String::from_utf8_lossy(&error_vec).to_string();
        Ok(ConnectResponse::Error(error_string))
    } else {
        Err(Error::IncorrectAction)
    }
}

fn get_announce_request(
    connection_id: i64,
    transaction_id: i32,
    listening_port: u16,
    info_hash: &[u8],
    peer_id: &[u8],
) -> Vec<u8> {
    use std::io::Write;
    assert!(info_hash.len() == 20);
    assert!(peer_id.len() == 20);

    let mut writer = vec![];
    writer.write_i64::<BigEndian>(connection_id).unwrap(); // connection_id
    writer.write_i32::<BigEndian>(ACTION_ANNOUNCE).unwrap(); // action
    writer.write_i32::<BigEndian>(transaction_id).unwrap(); // transaction_id

    Write::write(&mut writer, info_hash).unwrap(); // info_hash: 20 bytes
    Write::write(&mut writer, peer_id).unwrap(); // peer_id: 20 bytes

    writer.write_i64::<BigEndian>(0).unwrap(); // downloaded
    writer.write_i64::<BigEndian>(0).unwrap(); // left
    writer.write_i64::<BigEndian>(0).unwrap(); // uploaded
    writer.write_i32::<BigEndian>(EVENT_STARTED).unwrap(); // event
    writer.write_u32::<BigEndian>(0).unwrap(); // ip
    writer.write_u32::<BigEndian>(get_random_key()).unwrap(); // key
    writer.write_i32::<BigEndian>(30).unwrap(); // num_want
    writer.write_u16::<BigEndian>(listening_port).unwrap(); // port
    writer.write_u16::<BigEndian>(0).unwrap(); // extensions
    writer
}

fn read_announce_response(
    buf: &[u8],
    bytes_read: usize,
) -> Result<AnnounceResponse, std::io::Error> {
    let mut reader = std::io::Cursor::new(buf);
    let action = reader.read_i32::<BigEndian>().unwrap();
    let recv_transaction_id = reader.read_i32::<BigEndian>().unwrap();

    if action == ACTION_ANNOUNCE {
        let interval = reader.read_i32::<BigEndian>().unwrap();
        let num_leechers = reader.read_i32::<BigEndian>().unwrap();
        let num_seeders = reader.read_i32::<BigEndian>().unwrap();
        let mut peers = vec![];

        let mut bytes_left = bytes_read - reader.position() as usize;
        while bytes_left >= Peer::size() {
            let ip = reader.read_u32::<BigEndian>().unwrap();
            let port = reader.read_u16::<BigEndian>().unwrap();
            peers.push(Peer { ip: ip, port: port });
            bytes_left = bytes_read - reader.position() as usize;
        }
        assert!(bytes_left == 0);
        let res = AnnounceResponsePayload {
            transaction_id: recv_transaction_id,
            interval: std::time::Duration::from_secs(interval as u64),
            num_leechers: num_leechers,
            num_seeders: num_seeders,
            peers: peers,
        };
        Ok(AnnounceResponse::Payload(res))
    } else if action == ACTION_ERROR {
        let mut error_vec = vec![0u8; bytes_read - 8];
        std::io::Read::read_exact(&mut reader, &mut error_vec).unwrap();
        let error_string = String::from_utf8_lossy(&error_vec).to_string();
        Ok(AnnounceResponse::Error(error_string))
    } else {
        error!("Received invalid action {}", action);
        Err(std::io::Error::new(std::io::ErrorKind::Other, ""))
    }
}

fn get_scrape_request(connection_id: i64, transaction_id: i32) -> Vec<u8> {
    let mut writer = vec![];
    writer.write_i64::<BigEndian>(connection_id).unwrap(); // connection_id
    writer.write_i32::<BigEndian>(ACTION_SCRAPE).unwrap(); // action
    writer.write_i32::<BigEndian>(transaction_id).unwrap(); // transaction_id
    writer
}
