use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    pub length: u64,
    pub md5sum: Option<String>,
    pub path: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoDict {
    pub files: Option<Vec<FileInfo>>,
    pub length: Option<usize>,
    pub md5sum: Option<String>,
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
    pub private: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaInfo {
    pub announce: String,
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
    pub comment: Option<String>,
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
    #[serde(rename = "creation date")]
    pub creation_date: Option<u64>,
    pub encoding: Option<String>,
    pub info: InfoDict,
}
