use crate::utils;
use crate::clients;

use std::error::Error;
use serde_json::Value;
use super::torrent_spec::{self};

// Define function types for encoding and decoding
type EncoderFn = dyn Fn(&Value) -> Result<Vec<u8>, Box<dyn Error>>;
type DecoderFn = dyn Fn(&[u8], bool) -> Result<(Value, &[u8]), Box<dyn Error>>;

// TorrentManager struct to manage torrent-related functionalities
pub struct TorrentManager<'a> {
    encoder: &'a EncoderFn,  // Encoder function reference
    decoder: &'a DecoderFn,  // Decoder function reference
    metainfo: Option<torrent_spec::meta_info::Metainfo>,  // Optional Metainfo
    tracker_client: Option<clients::tracker_client::TrackerClient>,  // Optional TrackerClient
    peers: Option<Vec<torrent_spec::peer_info::Peer>>,  // Optional vector of Peers
}

impl<'a> TorrentManager<'a> {
    // Constructor for TorrentManager
    pub fn new(encoder: &'a EncoderFn, decoder: &'a DecoderFn) -> Self {
        Self { 
            encoder, 
            decoder, 
            metainfo: None, 
            tracker_client: None, 
            peers: None
        }
    }

    // Parses the meta info file from a byte vector
    pub fn parse_meta_info_file(&mut self, data: Vec<u8>) -> Result<(), Box<dyn Error>> {
        // Decode the data using the decoder function
        let decoded_value = (self.decoder)(&data, false)?.0;
        let mut metainfo: torrent_spec::meta_info::Metainfo = torrent_spec::meta_info::Metainfo::new();

        // Set various metainfo fields from the decoded data
        metainfo.set_tracker_url(utils::decode_base64_to_utf8_string(decoded_value["announce"].as_str().unwrap()).unwrap());
        metainfo.set_length(decoded_value["info"]["length"].as_i64().unwrap());
        metainfo.set_piece_length(decoded_value["info"]["piece length"].as_i64().unwrap());

        // Split the piece hash string into 40-character chunks
        let pieces_str = decoded_value["info"]["pieces"].as_str().unwrap();
        let piece_hashes: Vec<String> = pieces_str
            .as_bytes()
            .chunks(40)
            .map(|chunk| String::from_utf8(chunk.to_vec()).unwrap())
            .collect();
        metainfo.set_piece_hashes(piece_hashes);
        
        // Encode the info data and calculate its SHA1 hash
        let info_data = &decoded_value["info"];
        let encoded_info = (self.encoder)(info_data)?;
        let hash = utils::calculate_sha1_hash(encoded_info);
        metainfo.set_hash(hash);

        // Set the parsed metainfo to the struct
        self.metainfo = Some(metainfo);
        
        Ok(())
    }

    // Initialize clients such as TrackerClient and PeerClient
    pub fn init_clients(&mut self) -> Result<(), Box<dyn Error>> {
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
    pub async fn perform_peer_handshake(&self, peer_address: &String) -> Result<Vec<u8>, Box<dyn Error>>  {
        let mut peer_client = clients::peer_client::PeerClient::new();
        let info_hash = self.metainfo.as_ref().unwrap().get_hash().as_ref().unwrap().clone();
        let info_hash_bytes = utils::hex_to_byte_representation(&info_hash);

        peer_client.connect(peer_address).await?;
        let resp = peer_client.perform_handshake(info_hash_bytes).await;

        resp
    }

    // Print the list of peers
    pub fn print_peers(&self) -> Result<(), Box<dyn Error>> {
        if self.peers.is_none() {
            return Err("Error: peers were not initialized!".into());
        }

        for peer in self.peers.as_ref().unwrap() {
            println!("{}", peer.get_ip_address());
        }

        Ok(())
    }

    // Print the meta info
    pub fn print_meta_info(&self) -> Result<(), Box<dyn Error>> {
        self.is_meta_info_ok()?;
        println!("{}", self.metainfo.as_ref().unwrap().get_formatted_info());
        Ok(())
    }

    // Check if the meta info is initialized
    fn is_meta_info_ok(&self) -> Result<(), Box<dyn Error>> {
        if self.metainfo.is_none() {
            return Err("Error: meta info was not initialized!".into());
        }
        Ok(())
    }

    // Download the entire file by downloading each piece
    pub async fn download_file(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut file = vec![];
        let piece_hashes = self.metainfo.as_ref().unwrap().get_piece_hashes().clone().unwrap();
        let piece_count = piece_hashes.len();

        for piece_index in 0..piece_count {
            let piece = self.download_piece_with_index(piece_index as u32).await.unwrap();
            file.extend_from_slice(&piece);
        }
        Ok(file)
    }

    // Download a piece of the file with a specific index
    pub async fn download_piece_with_index(&self, piece_index: u32) -> Result<Vec<u8>, Box<dyn Error>> {
        let peer = &self.peers.as_ref().unwrap()[1];
        let piece_hashes = self.metainfo.as_ref().unwrap().get_piece_hashes().clone().unwrap();
        let piece_count = piece_hashes.len();

        // Determine the length of the piece
        let piece_length = if piece_index as usize == piece_count - 1 {
            let length = self.metainfo.as_ref().unwrap().get_length().unwrap();
            let piece_length = self.metainfo.as_ref().unwrap().get_piece_length().unwrap();
            let remainder = length % piece_length;
            if remainder == 0 { piece_length } else { remainder }
        } else {
            self.metainfo.as_ref().unwrap().get_piece_length().unwrap()
        };

        self.download_piece(peer.get_ip_address(), piece_index, piece_length as u32, &piece_hashes[piece_index as usize]).await
    }

    // Download a piece of the file from a peer
    pub async fn download_piece(&self, peer_address: &String, piece_index: u32, piece_length: u32, piece_hash: &String) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut peer_client = clients::peer_client::PeerClient::new();
        peer_client.connect(peer_address).await?;

        let info_hash = self.metainfo.as_ref().unwrap().get_hash().as_ref().unwrap().clone();
        let info_hash_bytes = utils::hex_to_byte_representation(&info_hash);

        println!("Performing handshake...");
        peer_client.perform_handshake(info_hash_bytes.clone()).await?;

        let mut piece = vec![];
        let block_size = 16 * 1024;
        let num_full_blocks = piece_length / block_size;
        let last_block_length = piece_length % block_size;

        peer_client.init_download().await?;

        // Download all full blocks
        for block_index in 0..num_full_blocks {
            let block = peer_client.download_block(piece_index, block_size * block_index, block_size).await?;
            piece.extend_from_slice(&block.2);
        }
        // Download the last block
        let last_block = peer_client.download_block(piece_index, num_full_blocks * block_size, last_block_length).await?;
        piece.extend_from_slice(&last_block.2);

        // Verify the piece hash
        let got_piece_hash = utils::calculate_sha1_hash_with_ref(&piece);
        if &got_piece_hash != piece_hash {
            return Err("Hash did not match!".into());
        }

        peer_client.disconnect().await?;

        return Ok(piece)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use base64::{engine::general_purpose, Engine};

    fn mock_encoder(value: &Value) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(serde_json::to_vec(value)?)
    }

    fn mock_decoder(data: &[u8], _: bool) -> Result<(Value, &[u8]), Box<dyn Error>> {
        let value: Value = serde_json::from_slice(data)?;
        Ok((value, &data[data.len()..]))
    }

    #[test]
    fn test_parse_meta_info_file() {
        let mut manager = TorrentManager::new(&mock_encoder, &mock_decoder);
        let data = json!({
            "announce": general_purpose::STANDARD.encode("http://tracker.example.com/announce"),
            "info": {
                "length": 12345,
                "piece length": 512,
                "pieces": general_purpose::STANDARD.encode("piecehashesexample")
            }
        }).to_string().into_bytes();

        assert!(manager.parse_meta_info_file(data).is_ok());
    }

}
