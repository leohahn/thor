use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    pub length: u64,
    pub md5sum: String,
    pub path: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoDict {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "announce-list", skip_serializing_if = "Option::is_none")]
    pub announce_list: Option<Vec<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "created by", skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(rename = "creation date", skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    pub info: InfoDict,
}
