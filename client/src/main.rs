extern crate bencoding;
extern crate env_logger;
extern crate serde;
extern crate sha1;
extern crate thor;

use serde::{Deserialize, Serialize};
use sha1::Digest;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
struct FileInfo {
    length: u64,
    md5sum: String,
    path: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InfoDict {
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<Vec<FileInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    md5sum: Option<String>,
    name: String,
    #[serde(rename = "piece length")]
    piece_length: u64,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>,
    private: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MetaInfo {
    announce: String,
    #[serde(rename = "announce-list", skip_serializing_if = "Option::is_none")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(rename = "created by", skip_serializing_if = "Option::is_none")]
    created_by: Option<String>,
    #[serde(rename = "creation date", skip_serializing_if = "Option::is_none")]
    creation_date: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding: Option<String>,
    info: InfoDict,
}

fn make_tracker_request(meta_info: &MetaInfo) -> Result<(), String> {
    let info_bytes = bencoding::to_bytes(&meta_info.info)
        .map_err(|e| format!("Failed to encode info dictionary: {}", e))?;
    {
        use std::io::Write;
        let mut file = std::fs::File::create("MY_FILE.txt").unwrap();
        let _ = file.write(&info_bytes).unwrap();
    }

    let mut hasher = sha1::Sha1::default();
    hasher.input(info_bytes);

    {
        print!("Info hash: ");
        let bytes = &hasher.result();
        assert!(bytes.len() == 20);
        for byte in bytes {
            print!("{:02x}", byte);
        }
        println!();
    }

    if meta_info.announce.starts_with("udp://") {
        Ok(())
    } else {
        Err("Currently only UDP is supported for trackers".to_owned())
    }
}

fn main() -> Result<(), String> {
    env_logger::init();

    let torrent_file = std::env::args().nth(1).unwrap();
    println!("Will parse torrent file {}", torrent_file);

    let mut torrent_file_bytes = vec![];
    let mut file = std::fs::File::open(torrent_file).unwrap();
    let _ = file.read_to_end(&mut torrent_file_bytes).unwrap();

    let meta_info: MetaInfo = bencoding::from_bytes(&torrent_file_bytes).unwrap();
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

    make_tracker_request(&meta_info)
}
