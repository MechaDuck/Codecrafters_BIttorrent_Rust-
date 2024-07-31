#![allow(dead_code)]

use std::net::Ipv4Addr;

use base64::{engine::general_purpose, Engine};
use anyhow::{anyhow, Ok, Result};
use sha1::{Sha1, Digest};

pub fn decode_base64_to_utf8_string(base64_string: &str) -> Result<String> {
    let bytes_string = general_purpose::STANDARD.decode(base64_string).map_err(|e| anyhow!(e))?;
    let utf8_string = std::str::from_utf8(&bytes_string).map_err(|e| anyhow!(e))?;
    Ok(utf8_string.to_string())
}

pub fn decode_base64_to_hex(base64_string: &str) -> Result<String> {
    let bytes_string = general_purpose::STANDARD.decode(base64_string).map_err(|e| anyhow!(e))?;
    let hex_string = hex::encode(bytes_string); 
    Ok(hex_string)
}

// Calculates sha1 hash from binary and returns it hex encoded
pub fn calculate_sha1_hash(data:Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    let hex_string = hex::encode(result);
    hex_string
}

// Calculates sha1 hash from binary and returns it hex encoded
pub fn calculate_sha1_hash_with_ref(data: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    let hex_string = hex::encode(result);
    hex_string
}

pub fn extract_peers_from_base64_string(peers_base64: String) -> Result<Vec<String>> {
    let decoded = general_purpose::STANDARD.decode(peers_base64).map_err(|e| anyhow!(e))?;

    let mut result = Vec::new();
    for chunk in decoded.chunks_exact(6) {
        if chunk.len() == 6 {
            // Extract the IP address
            let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
            // Extract the port
            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
            // Format as "IP:Port"
            let formatted = format!("{}:{}", ip, port);
            // Add to the result vector
            result.push(formatted);
        }
    }
    
    Ok(result)
    
}

pub fn hex_to_byte_representation(data: &String) -> Vec<u8> {

    let hex_string = hex::decode(data);
    hex_string.unwrap()
}

pub fn byte_vector_to_utf8_string(byte_vector: Vec<u8>) -> Result<String> {
    let utf8_string = String::from_utf8(byte_vector).unwrap();
    Ok(utf8_string)
}