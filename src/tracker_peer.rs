use std::collections::HashMap;

use reqwest::Url;

use crate::torrent_spec::TorrentMeta;

pub struct Peers {}

impl Peers {
    pub fn retrieve_peers(&mut self, t_info: TorrentMeta) -> Peers {
        let peer_data_fetch_url =
            Url::parse_with_params(&t_info.announce, Self::generate_params_for_peers(&t_info));
        Peers {}
    }

    fn generate_params_for_peers(t_info: &TorrentMeta) -> HashMap<String, String> {
        HashMap::from([
            ("info_hash".to_string(), t_info.info_hash.unwrap()),
            ("peer_id".to_string(), "".to_string()),
            ("port".to_string(), "".to_string()),
            ("uploaded".to_string(), "".to_string()),
            ("downloaded".to_string(), "".to_string()),
            ("left".to_string(), "".to_string()),
            ("event".to_string(), "".to_string()),
        ])
    }
}
