use crate::bencode_parser::BenStruct;

#[derive(Debug)]
pub struct TorrentMeta {
    pub announce: String,
    pub info: TorrentInfo,
}

#[derive(Debug)]
pub struct TorrentInfo {
    pub file_length: TorrentFileType,
    pub piece_length: usize,
    pub name: Option<String>,
    pub pieces: String,
}

#[derive(Debug)]
pub struct FileFormat {
    pub length: usize,
    pub path: Vec<String>,
}

#[derive(Debug)]
pub enum TorrentFileType {
    Length(usize),
    Files(Vec<FileFormat>),
}

impl TorrentMeta {
    pub fn extract_from_bcode(raw_bcode: BenStruct) -> TorrentMeta {
        if let BenStruct::Dict { data: ben_hashmap } = raw_bcode {
            println!("{:#?}", ben_hashmap);
            TorrentMeta {
                announce: ben_hashmap.get("announce".to_string()).unwrap(),
                info: TorrentInfo {
                    piece_length: ben_hashmap.get("length").unwrap(),
                },
            }
        } else {
            panic!("Benstruct returned isn't a dictionary.")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::bencode_parser::BencodeParser;

    use super::*;

    #[test]
    fn should_extract_meta() {
        let bcode = BencodeParser::new_w_file(Path::new("archive/deb.torrent")).decode_bencode();
        let extracted_meta = TorrentMeta::extract_from_bcode(bcode);
    }
}
