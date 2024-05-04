use crate::torrent_spec::TorrentMeta;

pub struct Peers {}

impl Peers {
    pub fn retrieve_peers(&mut self, t_info: TorrentMeta) -> Peers {
        let peer_data = reqwest::get(t_info.announce).form();
        Peers {}
    }
}
