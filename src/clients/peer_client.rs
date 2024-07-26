use reqwest::Client;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;

pub struct PeerClient {
    client: Client,
}


impl Default for PeerClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl PeerClient {

    pub fn new() -> Self {
        Self{ ..Default::default()}
    }
    
    pub async fn perform_handshake(&self, peer_address: &str, mut info_hash: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        // Connect to a peer
        let mut stream = TcpStream::connect(peer_address).await?;
    
        let mut handshake_message: Vec<u8> = Vec::new();
        let number: u8 = 19;
        handshake_message.push(number);
        handshake_message.extend_from_slice("BitTorrent protocol".as_bytes());
        handshake_message.extend_from_slice(&[0u8; 8]);
        handshake_message.extend_from_slice(&mut info_hash);
        handshake_message.extend_from_slice(&mut b"00112233445566778899".to_vec());
        // Write some data.
        stream.write_all(&handshake_message).await?;

        let len = 68;
        let mut buffer = vec![0; len];

        stream.read_exact(&mut buffer).await?;
    
        Ok(buffer)

    } 
}