use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use log::{debug, error, info, trace};
use rand::Rng;
use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::time::Duration;
use tokio::io;
use tokio::net::UdpSocket;
use tokio::prelude::*;

const MAGIC_CONSTANT: i64 = 0x41727101980;
const RECV_BUF_SIZE: usize = 1024;

const ACTION_CONNECT: i32 = 0;
const ACTION_ANNOUNCE: i32 = 1;
const ACTION_SCRAPE: i32 = 2;
const ACTION_ERROR: i32 = 3;

const EVENT_NONE: i32 = 0;
const EVENT_COMPLETED: i32 = 1;
const EVENT_STARTED: i32 = 2;
const EVENT_STOPPED: i32 = 3;

#[derive(Debug)]
pub struct Connection {
    addr: SocketAddr,
    socket: UdpSocket,
    id: i64,
}

#[derive(Debug)]
struct ConnectResponse {
    transaction_id: i32,
    action: i32,
    connection_id: i64,
}

#[derive(Debug)]
#[repr(C, packed)]
struct Peer {
    ip: i32,
    port: u16,
}

#[derive(Debug)]
struct AnnounceResponsePayload {
    transaction_id: i32,
    interval: std::time::Duration,
    num_leechers: i32,
    num_seeders: i32,
    peers: Vec<Peer>,
}

#[derive(Debug)]
enum AnnounceResponse {
    Payload(AnnounceResponsePayload),
    Error(String),
}

#[derive(Debug)]
pub enum Error {
    Tokio(tokio::io::Error),
    PortsExhausted,
    IncorrectTransactionId,
    IncorrectAction,
    Timeout,
    Server(String),
}

impl Connection {
    pub async fn new(addr: SocketAddr) -> Result<Connection, Error> {
        if let Some(mut socket) = try_bind_socket().await {
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
            })
        } else {
            Err(Error::PortsExhausted)
        }
    }

    pub async fn announce(&mut self) -> Result<(), Error> {
        let transaction_id = get_transaction_id();
        let info_hash = [0u8; 20];
        let peer_id = [0u8; 20];
        let announce_req =
            get_announce_request(self.id, transaction_id, 8080, &info_hash, &peer_id);

        self.socket
            .send(&announce_req)
            .await
            .map_err(Error::Tokio)?;

        let mut buf = [0u8; RECV_BUF_SIZE];

        let len = self
            .socket
            .recv(&mut buf)
            .timeout(Duration::from_secs(2))
            .await
            .map_err(|e| {
                error!("attempt to receive announce response timed out: {}", e);
                Error::Timeout
            })?
            .map_err(|e| {
                error!("receive packet failed: {}", e);
                Error::Tokio(e)
            })?;
        debug!("read {} bytes from dgram", len);

        let announce_res = read_announce_response(&buf, len)
            .map_err(|e| Error::Tokio(io::Error::new(io::ErrorKind::InvalidData, e)))?;

        match announce_res {
            AnnounceResponse::Payload(res) => {
                if res.transaction_id != transaction_id {
                    return Err(Error::Server(
                        "received incorrect transaction id".to_owned(),
                    ));
                }
                println!("leechers: {}", res.num_leechers);
                println!("seeders: {}", res.num_seeders);
                println!("interval: {:?}", res.interval);
                println!("peers (len = {}): {:?}", res.peers.len(), res.peers);
            }
            AnnounceResponse::Error(s) => return Err(Error::Server(s)),
        };

        Ok(())
    }
}

async fn try_bind_socket() -> Option<UdpSocket> {
    let port_range: RangeInclusive<i32> = 6881..=6889;
    for port in port_range {
        let local_addr: SocketAddr = format!("0.0.0.0:{}", port).parse().ok()?;
        let socket = UdpSocket::bind(&local_addr)
            .await
            .map_err(|e| error!("failed to bind to socket: {}", e))
            .ok()?;
        return Some(socket);
    }
    None
}

async fn connect(socket: &mut UdpSocket) -> Result<i64, Error> {
    let transaction_id = get_transaction_id();
    let connect_req = get_connect_request(transaction_id);

    socket.send(&connect_req).await.map_err(Error::Tokio)?;

    let mut buf = [0u8; RECV_BUF_SIZE];

    let len = socket
        .recv(&mut buf)
        .timeout(Duration::from_secs(2))
        .await
        .map_err(|e| {
            error!("attempt to connect timed out: {}", e);
            Error::Timeout
        })?
        .map_err(|e| {
            error!("receive packet failed: {}", e);
            Error::Tokio(e)
        })?;
    debug!("read {} bytes from dgram", len);

    let conn_res = read_connect_response(&buf)
        .map_err(|e| Error::Tokio(io::Error::new(io::ErrorKind::InvalidData, e)))?;

    if conn_res.transaction_id != transaction_id {
        error!("received incorrect transaction id");
        return Err(Error::IncorrectTransactionId);
    }

    if conn_res.action != ACTION_CONNECT {
        error!("received incorrect action");
        return Err(Error::IncorrectAction);
    }

    Ok(conn_res.connection_id)
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

fn read_connect_response(buf: &[u8]) -> Result<ConnectResponse, std::io::Error> {
    let mut reader = std::io::Cursor::new(buf);
    let action = reader.read_i32::<BigEndian>()?;
    let recv_transaction_id = reader.read_i32::<BigEndian>()?;
    let connection_id = reader.read_i64::<BigEndian>()?;
    let res = ConnectResponse {
        action: action,
        connection_id: connection_id,
        transaction_id: recv_transaction_id,
    };
    Ok(res)
}

fn get_announce_request(
    connection_id: i64,
    transaction_id: i32,
    listening_port: u16,
    info_hash: &[u8; 20],
    peer_id: &[u8; 20],
) -> Vec<u8> {
    use std::io::Write;
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
    writer.write_i32::<BigEndian>(-1).unwrap(); // num_want
    writer.write_u16::<BigEndian>(listening_port).unwrap(); // port
    writer.write_u16::<BigEndian>(0).unwrap(); // extensions
    writer
}

fn read_announce_response(
    buf: &[u8],
    bytes_read: usize,
) -> Result<AnnounceResponse, std::io::Error> {
    use std::mem::size_of;
    assert!(size_of::<Peer>() == size_of::<u16>() + size_of::<i32>());

    let mut reader = std::io::Cursor::new(buf);
    let action = reader.read_i32::<BigEndian>().unwrap();
    let recv_transaction_id = reader.read_i32::<BigEndian>().unwrap();

    if action == ACTION_ANNOUNCE {
        let interval = reader.read_i32::<BigEndian>().unwrap();
        let num_leechers = reader.read_i32::<BigEndian>().unwrap();
        let num_seeders = reader.read_i32::<BigEndian>().unwrap();
        let mut peers = vec![];

        let mut bytes_left = bytes_read - reader.position() as usize;
        while bytes_left >= size_of::<Peer>() {
            let ip = reader.read_i32::<BigEndian>().unwrap();
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
        let mut error_vec = vec![];
        std::io::Read::read_to_end(&mut reader, &mut error_vec).unwrap();
        let error_string = String::from_utf8_lossy(&error_vec).to_string();
        Ok(AnnounceResponse::Error(error_string))
    } else {
        error!("Received invalid action {}", action);
        Err(std::io::Error::new(std::io::ErrorKind::Other, ""))
    }
}
