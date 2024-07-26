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