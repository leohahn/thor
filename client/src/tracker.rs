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
const ACTION_CONNECT: i32 = 0;

pub struct Connection {
    addr: SocketAddr,
    socket: UdpSocket,
}

pub enum Error {
    Tokio(tokio::io::Error),
    PortsExhausted,
    IncorrectTransactionId,
    IncorrectAction,
    Timeout,
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
            })
        } else {
            Err(Error::PortsExhausted)
        }
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

struct ConnectResponse {
    transaction_id: i32,
    action: i32,
    connection_id: i64,
}

async fn connect(socket: &mut UdpSocket) -> Result<i64, Error> {
    let transaction_id = get_transaction_id();
    let connect_req = get_connect_request(transaction_id);

    socket.send(&connect_req).await.map_err(Error::Tokio)?;

    let mut buf = vec![];

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
    trace!("read {} bytes from dgram", len);

    let conn_res = read_connect_response(&buf, transaction_id)
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

fn get_connect_request(transaction_id: i32) -> Vec<u8> {
    // protocol id
    let mut writer = vec![];
    writer.write_i64::<BigEndian>(MAGIC_CONSTANT).unwrap();
    writer.write_i32::<BigEndian>(ACTION_CONNECT).unwrap();
    writer.write_i32::<BigEndian>(transaction_id).unwrap();
    writer
}

fn read_connect_response(
    buf: &[u8],
    transaction_id: i32,
) -> Result<ConnectResponse, std::io::Error> {
    let mut reader = std::io::Cursor::new(buf);
    let action = reader.read_i32::<BigEndian>()?;
    let recv_transaction_id = reader.read_i32::<BigEndian>()?;
    let connection_id = reader.read_i64::<BigEndian>()?;
    let res = ConnectResponse {
        action: action,
        connection_id: connection_id,
        transaction_id: transaction_id,
    };
    Ok(res)
}
