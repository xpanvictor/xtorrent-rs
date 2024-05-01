use crate::bencode_parser::{encode_bencode, BenStruct, GetData};
use sha1::{Digest, Sha1};

#[derive(Debug)]
pub struct TorrentMeta {
    pub announce: String,
    pub info: TorrentInfo,
    pub info_hash: Option<[u8; 20]>,
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
            let info_struct_bc = torrent_hashmap.get("info").unwrap().to_owned();
            let info_struct = match &info_struct_bc {
                BenStruct::Dict { data } => data,
                _ => panic!("Invalid info dictionary in the torrent data"),
            };
            let is_folder = info_struct.get("files").is_some();

            let file_length = if is_folder {
                let base_vec: Vec<FileFormat> = if let BenStruct::List { data } =
                    info_struct.get("files").unwrap()
                {
                    data.iter()
                            .map(|file_struct| {
                                if let BenStruct::Dict { data: file_element } = file_struct {
                                    FileFormat {
                                        length: file_element.get("length").unwrap().get_isize(),
                                        path: if let BenStruct::List { data: path_list } =
                                            file_element.get("path").unwrap()
                                        {
                                            if path_list.is_empty() {panic!("Invalid torrent, path: a zero length list is an error case")}
                                            path_list
                                                .iter()
                                                .map(|path_element| path_element.get_string())
                                                .collect()
                                        } else {
                                            panic!("Invalid torrent, no path in file format")
                                        },
                                    }
                                } else {
                                    panic!("Invalid torrent, files element is not a dict")
                                }
                            })
                            .collect()
                } else {
                    // Note: This is redundant, you checked if is a folder already
                    panic!("Invalid torrent, files not found in a file project")
                };
                TorrentFileType::Files(base_vec)
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
            let mut hasher = Sha1::new();
            hasher.update(encode_bencode(&info_struct_bc));
            TorrentMeta {
                announce: announce_byte.get_string(),
                info: torrent_info,
                // Conversely that means clients must either reject invalid metainfo files or extract
                // the substring directly. They must not perform a decode-encode roundtrip on invalid data.
                // NOTE: I'm not checking key ordering
                info_hash: Some(hasher.finalize().into()),
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
        TorrentMeta::extract_from_bcode(bcode);
    }

    #[test]
    fn should_extract_meta_from_folder_torrent() {
        let bcode =
            BencodeParser::new_w_file(Path::new("archive/testtor.torrent")).decode_bencode();
        let extracted_meta = TorrentMeta::extract_from_bcode(bcode);
        println!("Crafted: {:?}", extracted_meta.info_hash);
    }
}
