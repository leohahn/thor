extern crate bencoding;
extern crate env_logger;
extern crate serde;

use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
struct FileInfo {
    length: u64,
    md5sum: String,
    path: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InfoDict {
    #[serde(rename = "piece length")]
    piece_length: u64,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>,
    private: Option<bool>,
    name: String,
    length: Option<usize>,
    md5sum: Option<String>,
    files: Option<Vec<FileInfo>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MetaInfo {
    info: InfoDict,
    announce: String,
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(rename = "creation date")]
    creation_date: Option<u64>,
    comment: Option<String>,
    #[serde(rename = "created by")]
    created_by: Option<String>,
    encoding: Option<String>,
}

fn main() {
    env_logger::init();

    let torrent_file = std::env::args().nth(1).unwrap();
    println!("Will parse torrent file {}", torrent_file);

    let mut torrent_file_bytes = vec![];
    let mut file = std::fs::File::open(torrent_file).unwrap();
    let _ = file.read_to_end(&mut torrent_file_bytes).unwrap();

    let meta_info: MetaInfo = bencoding::from_bytes(&torrent_file_bytes).unwrap();
    println!("Announce = {}", meta_info.announce);
}
