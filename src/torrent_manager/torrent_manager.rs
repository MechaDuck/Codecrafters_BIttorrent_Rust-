use crate::utils;
use crate::clients;

use std::error::Error;
use super::torrent_spec::{self};
use serde_json::Value;
use anyhow::{anyhow, Ok, Result};

// Define a function type for encoding
type EncoderFn = dyn Fn(&Value) -> Result<Vec<u8>>;

// Define a function type for decoding
type DecoderFn = dyn Fn(&[u8], bool) -> Result<(Value, &[u8])>;

pub struct TorrentManager<'a> {
    encoder: &'a EncoderFn,
    decoder: &'a DecoderFn,
    metainfo: Option<torrent_spec::meta_info::Metainfo>,
    tracker_client: Option<clients::tracker_client::TrackerClient>,
    peers : Option<Vec<torrent_spec::peer_info::Peer>>,
    peer_client: Option<clients::peer_client::PeerClient>,
}


impl<'a> TorrentManager<'a> {
    pub fn new(encoder: &'a EncoderFn, decoder: &'a DecoderFn) -> Self {
        
        TorrentManager { 
            encoder, 
            decoder, 
            metainfo: None, 
            peers: None,
            tracker_client: None,
            peer_client: None
        }
    }

    pub fn parse_meta_info_file(&mut self, data: Vec<u8>) -> Result<()>{
        let decoded_value = (self.decoder)(&data, false)?.0;
        let mut metainfo: torrent_spec::meta_info::Metainfo = torrent_spec::meta_info::Metainfo::new();

        metainfo.set_tracker_url(utils::decode_base64_to_utf8_string(decoded_value["announce"].as_str().unwrap()).unwrap());
        metainfo.set_length(decoded_value["info"]["length"].as_i64().unwrap());
        metainfo.set_piece_length(decoded_value["info"]["piece length"].as_i64().unwrap());
        metainfo.set_piece_hashes( utils::decode_base64_to_hex(decoded_value["info"]["pieces"].as_str().unwrap())?);
        
        let info_data = &decoded_value["info"];
        let encoded_info = (self.encoder)(info_data)?;
        let hash = utils::calculate_sha1_hash(encoded_info);
        metainfo.set_hash(hash);

        self.metainfo = Some(metainfo);
        
        Ok(())
    }

    pub fn init_clients(&mut self) -> Result<()> {
        self.is_meta_info_ok()?;

        let metainfo = self.metainfo.as_ref().unwrap();
        let tracker_url = metainfo.get_tracker_url().as_ref().unwrap().clone();
        let length = metainfo.get_length().as_ref().unwrap().clone();
        let info_hash = metainfo.get_hash().as_ref().unwrap().clone();

        self.tracker_client = Some(clients::tracker_client::TrackerClient::new(tracker_url));
        let resp = self.tracker_client.as_ref().unwrap().request_peers(length, info_hash).unwrap();
        let decoded_peer_info = (self.decoder)(&resp, false).unwrap().0;

        let extracted_peers = utils::extract_peers_from_base64_string(decoded_peer_info["peers"].as_str().unwrap().to_string()).unwrap();
        
        let mut peers_vector: Vec<torrent_spec::peer_info::Peer> = vec![];
        for peer in extracted_peers {
            peers_vector.push(torrent_spec::peer_info::Peer::new(peer));
        }
        self.peers = Some(peers_vector);

        Ok(())
    }

    pub async fn perform_peer_handshake(&self, peer_address: &String)  -> Result<Vec<u8>, Box<dyn Error>>{
        let peer_client = clients::peer_client::PeerClient::new();
        let info_hash = self.metainfo.as_ref().unwrap().get_hash().as_ref().unwrap().clone();
        let info_hash_bytes = utils::hex_to_byte_representation(&info_hash);
        let resp = peer_client.perform_handshake(peer_address, info_hash_bytes).await;
        resp
    }

    pub fn print_peers(&self) -> Result<()> {
        if self.peers.is_none(){
            return Err(anyhow!("Error: peers were not initialized!"));
        }

        for peer in self.peers.as_ref().unwrap(){
            print!("{}\n", peer.get_ip_address());
        }

        Ok(())
    }



    pub fn print_meta_info(&self) -> Result<()>{
        self.is_meta_info_ok()?;
        print!("{}", self.metainfo.as_ref().unwrap().get_formatted_info());
        Ok(())
    }

    fn is_meta_info_ok(&self) -> Result<()> {
        if self.metainfo.is_none(){
            return Err(anyhow!("Error: meta info was not initialized!"));
        }
        Ok(())
    }



}