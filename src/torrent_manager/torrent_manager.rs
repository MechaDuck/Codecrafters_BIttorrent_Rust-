use crate::utils;
use crate::clients;

use core::num;
use std::error::Error;
use super::torrent_spec::{self};
use serde_json::Value;
use anyhow::{anyhow, Ok, Result, Context};

// Define a function type for encoding
type EncoderFn = dyn Fn(&Value) -> Result<Vec<u8>>;

// Define a function type for decoding
type DecoderFn = dyn Fn(&[u8], bool) -> Result<(Value, &[u8])>;

// TorrentManager struct to manage torrent related functionalities
pub struct TorrentManager<'a> {
    encoder: &'a EncoderFn,  // Encoder function reference
    decoder: &'a DecoderFn,  // Decoder function reference
    metainfo: Option<torrent_spec::meta_info::Metainfo>,  // Optional Metainfo
    tracker_client: Option<clients::tracker_client::TrackerClient>,  // Optional TrackerClient
    peers: Option<Vec<torrent_spec::peer_info::Peer>>,  // Optional vector of Peers
    peer_client: Option<clients::peer_client::PeerClient>,  // Optional PeerClient
}

impl<'a> TorrentManager<'a> {
    // Constructor for TorrentManager
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

    // Parses the meta info file from a byte vector
    pub fn parse_meta_info_file(&mut self, data: Vec<u8>) -> Result<()> {
        // Decode the data using the decoder function
        let decoded_value = (self.decoder)(&data, false)?.0;
        let mut metainfo: torrent_spec::meta_info::Metainfo = torrent_spec::meta_info::Metainfo::new();

        // Set various metainfo fields from the decoded data
        metainfo.set_tracker_url(utils::decode_base64_to_utf8_string(decoded_value["announce"].as_str().unwrap()).unwrap());
        metainfo.set_length(decoded_value["info"]["length"].as_i64().unwrap());
        metainfo.set_piece_length(decoded_value["info"]["piece length"].as_i64().unwrap());
        metainfo.set_piece_hashes(utils::decode_base64_to_hex(decoded_value["info"]["pieces"].as_str().unwrap())?);
        
        let info_data = &decoded_value["info"];
        // Encode the info data and calculate its SHA1 hash
        let encoded_info = (self.encoder)(info_data)?;
        let hash = utils::calculate_sha1_hash(encoded_info);
        metainfo.set_hash(hash);

        // Set the parsed metainfo to the struct
        self.metainfo = Some(metainfo);
        
        Ok(())
    }

    // Initialize clients such as TrackerClient and PeerClient
    pub fn init_clients(&mut self) -> Result<()> {
        self.is_meta_info_ok()?;

        let metainfo = self.metainfo.as_ref().unwrap();
        let tracker_url = metainfo.get_tracker_url().as_ref().unwrap().clone();
        let length = metainfo.get_length().as_ref().unwrap().clone();
        let info_hash = metainfo.get_hash().as_ref().unwrap().clone();

        // Create a new TrackerClient and request peers
        self.tracker_client = Some(clients::tracker_client::TrackerClient::new(tracker_url));
        let resp = self.tracker_client.as_ref().unwrap().request_peers(length, info_hash).unwrap();
        let decoded_peer_info = (self.decoder)(&resp, false).unwrap().0;

        // Extract peers from the decoded peer info
        let extracted_peers = utils::extract_peers_from_base64_string(decoded_peer_info["peers"].as_str().unwrap().to_string()).unwrap();
        
        let mut peers_vector: Vec<torrent_spec::peer_info::Peer> = vec![];
        for peer in extracted_peers {
            peers_vector.push(torrent_spec::peer_info::Peer::new(peer));
        }
        self.peers = Some(peers_vector);

        Ok(())
    }

    // Perform handshake with a peer asynchronously
    pub async fn perform_peer_handshake(&self, peer_address: &String) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut peer_client = clients::peer_client::PeerClient::new();
        let info_hash = self.metainfo.as_ref().unwrap().get_hash().as_ref().unwrap().clone();
        let info_hash_bytes = utils::hex_to_byte_representation(&info_hash);
        peer_client.connect(peer_address).await?;
        let resp = peer_client.perform_handshake(info_hash_bytes).await;


        resp
    }

    // Print the list of peers
    pub fn print_peers(&self) -> Result<()> {
        if self.peers.is_none() {
            return Err(anyhow!("Error: peers were not initialized!"));
        }

        for peer in self.peers.as_ref().unwrap() {
            print!("{}\n", peer.get_ip_address());
        }

        Ok(())
    }

    // Print the meta info
    pub fn print_meta_info(&self) -> Result<()> {
        self.is_meta_info_ok()?;
        print!("{}", self.metainfo.as_ref().unwrap().get_formatted_info());
        Ok(())
    }

    // Check if the meta info is initialized
    fn is_meta_info_ok(&self) -> Result<()> {
        if self.metainfo.is_none() {
            return Err(anyhow!("Error: meta info was not initialized!"));
        }
        Ok(())
    }

    pub async fn download_file(&self)-> Result<Vec<u8>> {
        let mut file = vec![];
        let mut piece_hashes = self.metainfo.as_ref().unwrap().get_piece_hashes().clone().unwrap();
        for e in 0..piece_hashes.len()/40{
            let piece = self.download_piece_with_index(e as u32).await.unwrap();
            file.extend_from_slice(&piece);
        }
        Ok(file)
    }
    pub async fn download_piece_with_index(&self, piece_index: u32) -> Result<Vec<u8>>{

        let peer =  &self.peers.as_ref().unwrap()[1];

        let mut piece_hashes = self.metainfo.as_ref().unwrap().get_piece_hashes().clone().unwrap();

        println!("piece_hashes: {}, index: {}", piece_hashes.len(), piece_index);
        let mut piece_length;
        if piece_hashes.len()/40 -1 == piece_index as usize{
            piece_length = self.metainfo.as_ref().unwrap().get_length().clone().unwrap() % self.metainfo.as_ref().unwrap().get_piece_length().clone().unwrap();
            if piece_length == 0 {
                piece_length = self.metainfo.as_ref().unwrap().get_piece_length().clone().unwrap();
            }
        }else {
            piece_length = self.metainfo.as_ref().unwrap().get_piece_length().clone().unwrap();
        }

        return self.download_piece(peer.get_ip_address(), piece_index, piece_length as u32, 0.to_string()).await;

    }

    pub async fn download_piece(&self, peer_address: &String, piece_index: u32, piece_length: u32, piece_hash: String) -> Result<Vec<u8>> {
        let mut peer_client = clients::peer_client::PeerClient::new();
        peer_client.connect(peer_address).await;

        //let mut peer_client = clients::peer_client::PeerClient::new();
        let info_hash = self.metainfo.as_ref().unwrap().get_hash().as_ref().unwrap().clone();
        let info_hash_bytes = utils::hex_to_byte_representation(&info_hash);

        // TODO: Error handling for all awaits
        println!("Performing handshake...");
        peer_client.perform_handshake(info_hash_bytes.clone()).await;

        //TODO: Check responses from awaits

        // create workpackages
        // (piece_index, block_index, block_begin, block_length)

        let mut piece = vec![];
        let block_size = 16*1024;
        let num_full_blocks = piece_length / block_size;
        let last_block_length = piece_length % block_size;

        peer_client.init_download().await;

        for block_index in 0..num_full_blocks{
            //println!("Start download block from piece. Piece_index: {}, Current length: {}, Full length: {}  Offset: {}...",piece_index, piece.len(), piece_length,block_size * block_index);
            let block = peer_client.download_block(piece_index, block_size * block_index, block_size).await.unwrap();
            piece.extend_from_slice(&block.2);
        }
        let block = peer_client.download_block(piece_index, num_full_blocks * block_size, last_block_length).await.unwrap();
        piece.extend_from_slice(&block.2);

        let got_piece_hash = utils::calculate_sha1_hash_with_ref(&piece);

        // if got_piece_hash != piece_hash {
        //     return Err(anyhow!("Hash did not match!"));
        // }
        peer_client.disconnect().await;

        println!("Downloaded successfully. Length: {}, Size: {}", piece_length, piece.len());
        Ok(piece)

    }
}
