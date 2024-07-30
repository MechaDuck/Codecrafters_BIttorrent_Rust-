use reqwest::Client;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;

use crate::utils;

pub struct PeerClient {
    stream: Option<TcpStream>,
}


impl Default for PeerClient {
    fn default() -> Self {
        Self {
            stream: None,
        }
    }
}

impl PeerClient {

    pub fn new() -> Self {
        Self{ ..Default::default()}
    }

    pub async fn connect(&mut self, peer_address: &str) -> Result<(), Box<dyn Error>>{
        // Connect to a peer
        self.stream = Some(TcpStream::connect(peer_address).await?);
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(mut stream) = self.stream.take() {
            stream.shutdown().await?;
        }
        Ok(())
    }

    fn ensure_connected(&mut self) -> Result<&mut TcpStream, Box<dyn Error>> {
        match self.stream.as_mut() {
            Some(stream) => Ok(stream),
            None => Err("Not connected to a peer".into()),
        }
    }

    
    
    pub async fn perform_handshake(&mut self, mut info_hash: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        let stream = self.ensure_connected()?;
        stream.flush();
    
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

    async fn read_message(&mut self) -> Result<(u8, Vec<u8>), Box<dyn Error>> {
        let stream = self.ensure_connected()?;

        // Read the message length prefix (4 bytes)
        let mut length_prefix = [0u8; 4];
        stream.read_exact(&mut length_prefix).await?;
        let message_length = u32::from_be_bytes(length_prefix);

        // Read the rest of the message
        let mut message = vec![0u8; message_length as usize];
        stream.read_exact(&mut message).await?;

        // The first byte of the message is the message ID
        let message_id = message[0];
        let payload = message[1..].to_vec();

        Ok((message_id, payload))
    }

    async fn send_message(&mut self, message_id: u8, payload:  Vec<u8>) -> Result<(), Box<dyn Error>> {
        let stream = self.ensure_connected()?;
        
        let length_prefix = ((payload.len() + 1) as u32).to_be_bytes();
        
        let mut message = vec![];
        message.extend_from_slice(&length_prefix);
        message.push(message_id);
        message.extend_from_slice(&payload);

        stream.write_all( &message).await?;

        Ok(())
    }

    pub async fn wait_for_message(&mut self) -> Result<(u8, Vec<u8>), Box<dyn Error>> {
        let (message_id, payload) = self.read_message().await?;

        match message_id {
            0 => println!("Received choke message"),
            1 => println!("Received unchoke message"),
            2 => println!("Received interested message"),
            3 => println!("Received not interested message"),
            4 => println!("Received have message"),
            5 => println!("Received bitfield message"),
            6 => println!("Received request message"),
            7 => println!("Received piece message"),
            8 => println!("Received cancel message"),
            9 => println!("Received port message"),
            _ => println!("Received unknown message with ID: {}", message_id),
        }

        Ok((message_id, payload))
    }
    pub async fn init_download(&mut self) -> Result<(), Box<dyn Error>>{
        let (message_id, _) = self.wait_for_message().await?;
        if message_id != 5{
            return Err("Could not find message id: \"bitfield\"".into())
        }
        self.send_message(2, vec![]).await?;

        let (message_id, _) = self.wait_for_message().await?;
        if message_id != 1{
            return Err("Could not find message id: \"unchoke\"".into())
        }

        Ok(())
    }
    pub async fn download_block(&mut self, index: u32, begin: u32, length: u32) -> Result<(u32,u32,Vec<u8>), Box<dyn Error>>{
    

        let mut payload = Vec::new();
        payload.extend_from_slice(&index.to_be_bytes());
        payload.extend_from_slice(&begin.to_be_bytes());
        payload.extend_from_slice(&length.to_be_bytes());
        self.send_message(6, payload).await?;

        // Wait for a piece message (message_id = 7)
        let (message_id, payload) = self.wait_for_message().await?;
        if message_id != 7 {
            return Err("Could not find message id: \"piece message\"".into());
        }

        // Extract the index, begin, and block from the payload
        let index_from_payload = u32::from_be_bytes(payload[0..4].try_into().unwrap());
        let begin_from_payload = u32::from_be_bytes(payload[4..8].try_into().unwrap());
        let block = payload[8..8 + length as usize].to_vec();

        Ok((index_from_payload, begin_from_payload, block))
    }




}