use std::collections::HashMap;
use std::{fs, path::PathBuf, str::FromStr};

enum FileDescriptor {
    SingleFile,
    MultiFile,
}

struct MetaInfo {
    info: FileDescriptor,
}

fn read_torrent_file<T>(reader: &mut T)
where
    T: std::io::Read,
{
}

fn main() {
    let mut torrent_file =
        fs::File::open("C:/Users/leona/Downloads/1BE7C19BAB12B1BD7DF6AB47FFDF1DBFC6714E9D.torrent")
            .unwrap();
    read_torrent_file(&mut torrent_file);

    println!("Hello, world!");
}
