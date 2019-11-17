use crate::error::Error;
use log::{error, info};
use std::net::SocketAddr;
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Peer {
    pub ip: u32,
    pub port: u16,
}

impl std::fmt::Display for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let first = (self.ip as u32) >> 24;
        let second = ((self.ip as u32) & 0b00000000_11111111_00000000_00000000) >> 16;
        let third = ((self.ip as u32) & 0b00000000_00000000_11111111_00000000) >> 8;
        let fourth = (self.ip as u32) & 0b00000000_00000000_00000000_11111111;
        write!(f, "{}.{}.{}.{}:{}", first, second, third, fourth, self.port)
    }
}

impl Peer {
    pub fn size() -> usize {
        use std::mem::size_of;
        size_of::<u32>() + size_of::<u16>()
    }

    pub async fn start_connection(self) -> Result<(), Error> {
        let addr_str = self.to_string();
        let socket = addr_str
            .parse::<SocketAddr>()
            .map(|addr| TcpStream::connect(addr))?
            .await
            .map_err(|e| {
                error!("failed to connect to peer: {}", e);
                e
            })?;

        info!("successfuly connected to peer at {}", addr_str);

        loop {
            // TODO: add graceful shutdown here
        }
    }
}
