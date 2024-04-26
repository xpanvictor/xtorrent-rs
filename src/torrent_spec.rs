use crate::bencode_parser::{BenStruct, GetData};

#[derive(Debug)]
pub struct TorrentMeta {
    pub announce: String,
    pub info: TorrentInfo,
}

#[derive(Debug)]
pub struct TorrentInfo {
    pub file_length: TorrentFileType,
    pub piece_length: isize,
    pub name: Option<String>,
    pub pieces: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct FileFormat {
    pub length: isize,
    pub path: Vec<String>,
}

#[derive(Debug)]
pub enum TorrentFileType {
    Length(isize),
    Files(Vec<FileFormat>),
}

impl TorrentMeta {
    // # panics
    pub fn extract_from_bcode(raw_bcode: BenStruct) -> TorrentMeta {
        if let BenStruct::Dict {
            data: torrent_hashmap,
        } = raw_bcode
        {
            let announce_byte = torrent_hashmap.get("announce").unwrap().to_owned();
            let info_struct = torrent_hashmap.get("info").unwrap().to_owned();
            let info_struct = match info_struct {
                BenStruct::Dict { data } => data,
                _ => panic!("Invalid info dictionary in the torrent data"),
            };
            let is_folder = info_struct.get("files").is_some();

            let file_length = if is_folder {
                // TODO: to handle folder structure
                TorrentFileType::Files(vec![])
            } else {
                TorrentFileType::Length(info_struct.get("length").unwrap().get_isize())
            };

            let pieces: Vec<Vec<u8>> =
                if let BenStruct::Byte { data, length } = info_struct.get("pieces").unwrap() {
                    if length % 20 != 0 {
                        panic!("Invalid torrent specification passed")
                    }
                    data.chunks(20).map(|chunk| chunk.to_vec()).collect()
                } else {
                    panic!("Pieces not found or invalid")
                };

            // filling the info struct
            let torrent_info = TorrentInfo {
                piece_length: info_struct
                    .get("piece length")
                    .unwrap()
                    .to_owned()
                    .get_isize(),
                name: Some(info_struct.get("name").unwrap().get_string()),
                pieces,
                file_length,
            };
            TorrentMeta {
                announce: announce_byte.get_string(),
                info: torrent_info,
            }
        } else {
            panic!("Invalid bencode passed")
        }

        // use torrent info
    }
}

#[cfg(test)]
mod torrent_struct_tests {
    use std::path::Path;

    use crate::bencode_parser::BencodeParser;

    use super::*;

    #[test]
    fn should_extract_meta_from_single_file_torrent() {
        let bcode = BencodeParser::new_w_file(Path::new("archive/debr.torrent")).decode_bencode();
        let extracted_meta = TorrentMeta::extract_from_bcode(bcode);
        println!("Crafted: {:?}", extracted_meta);
    }

    #[test]
    fn should_extract_meta_from_folder_torrent() {
        let bcode =
            BencodeParser::new_w_file(Path::new("archive/testtor.torrent")).decode_bencode();
        let extracted_meta = TorrentMeta::extract_from_bcode(bcode);
        println!("Crafted: {:?}", extracted_meta);
    }
}
