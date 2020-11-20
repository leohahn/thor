extern crate bencoding;
extern crate env_logger;
extern crate futures_util;
extern crate serde;
extern crate sha1;
extern crate thor;
extern crate tokio;

use std::io::Read;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::TcpStream;

async fn peer_connection(addr: String) {
    let socket_addr: SocketAddr = addr.parse().unwrap();
    let socket = TcpStream::connect(&socket_addr).await.unwrap();
}

async fn make_tracker_request(meta_info: &thor::MetaInfo) -> Result<(), String> {
    println!("announce: {}", meta_info.announce);
    println!("announce_list: {:?}", meta_info.announce_list);

    // TODO: add support for the multi tracker extension

    if meta_info.announce.starts_with("udp://") {
        let (_, mut url) = meta_info.announce.split_at("udp://".len());

        if let Some(index) = url.rfind('/') {
            let (u, _) = url.split_at(index);
            url = u;
        }

        println!("url is: {}", url);

        let mut addrs_iter = url.to_socket_addrs().unwrap();
        if let Some(addr) = addrs_iter.next() {
            println!("resolved to ip {}", addr);
            let mut connection = thor::tracker::Connection::new(addr).await.unwrap();
            let res = connection.announce(&meta_info.info).await.unwrap();

            // TODO: use stable rust, since async await is already supported.

            for peer in res.peers {
                tokio::spawn(async move {
                    match peer.start_connection().await {
                        Ok(_) => {}
                        Err(e) => {
                            println!("connection with peer failed: {}", e);
                        }
                    };
                });
            }

            // TODO: pass channel to peer connections in order to manage and wait for them
            loop {}

            Ok(())
        } else {
            Err(format!("failed to resolve address {}", url))
        }
    } else {
        Err("Currently only UDP is supported for trackers".to_owned())
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::init();

    let torrent_file = std::env::args().nth(1).unwrap();

    {
        use sha1::Digest;
        let mut file = std::fs::File::open(&torrent_file).unwrap();
        let mut buf = vec![];
        file.read_to_end(&mut buf).unwrap();

        let mut hasher = sha1::Sha1::new();
        hasher.update(&buf);

        let bytes = &hasher.finalize();
        assert!(bytes.len() == 20);

        println!(":D info_hash: {:02x}", &bytes);
    }

    println!("Will parse torrent file {}", torrent_file);

    let mut torrent_file_bytes = vec![];
    let mut file = std::fs::File::open(torrent_file).unwrap();
    let _ = file.read_to_end(&mut torrent_file_bytes).unwrap();

    let meta_info: thor::MetaInfo = bencoding::from_bytes(&torrent_file_bytes).unwrap();
    println!("Tracker URL = {}", meta_info.announce);
    println!(
        "Piece length = {:.2} KiB",
        meta_info.info.piece_length as f32 / 1024.0
    );
    if let Some(l) = meta_info.info.length.as_ref() {
        println!("File length = {:.2} MiB", *l as f32 / (1024.0 * 1024.0));
    }
    println!("Num pieces = {}", meta_info.info.pieces.len() / 20);
    if let Some(files) = meta_info.info.files.as_ref() {
        println!("Directory to download = {}", meta_info.info.name);
        for f in files {
            assert!(f.path.len() > 0);
            println!("  Path: {}", f.path.join("/"));
            println!("  Size: {} MiB", f.length as f32 / (1024.0 * 1024.0));
        }
    } else {
        println!("File to download = {}", meta_info.info.name);
    }

    println!("info dict: {:?}", meta_info.info);
    println!(
        "info dict bencoded: {}",
        String::from_utf8_lossy(&bencoding::to_bytes(&meta_info.info).unwrap())
    );

    make_tracker_request(&meta_info).await
}
