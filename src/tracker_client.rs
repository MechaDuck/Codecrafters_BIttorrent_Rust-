use std::{collections::HashMap, error::Error};
use anyhow::anyhow;
use reqwest::blocking::Client;
use nanoid::nanoid;
use serde_urlencoded::to_string;
use hex;
pub struct TrackerClient {
    client: Client,
    root_url: String,
}
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

impl Default for TrackerClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
            root_url: "".to_string(),
        }
    }
}

impl TrackerClient {

    pub fn new(root_url: String) -> Self {
            Self{root_url, ..Default::default()}
    }

    pub fn request_peers(&self, length: i64, hex_info_hash: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut params = HashMap::new();
        
        let bytes_info_hash = hex::decode(hex_info_hash).unwrap();
        let url_encoded_info_hash = percent_encode(&bytes_info_hash, NON_ALPHANUMERIC).to_string();

        // query parameters
        params.insert("info_hash", url_encoded_info_hash.clone());
        params.insert("peer_id", nanoid!(20));
        params.insert("port", 6881.to_string());
        params.insert("uploaded", 0.to_string());
        params.insert("downloaded", 0.to_string());
        params.insert("left", length.to_string());
        params.insert("compact", 1.to_string());
        
        let mut query_string = String::new();
        for (key, value) in &params {
            if !query_string.is_empty() {
                query_string.push('&');
            }
            query_string.push_str(&format!("{}={}", key, value));
        }
        
        // Construct the full URL
        let request_url = format!("{}?{}", self.root_url, query_string);

        let response = self.client.get(request_url.clone()).send()?;


        if response.status().is_success() {
            let body_bytes = response.bytes().unwrap().to_vec();
            return Ok(body_bytes);
        }
        
        Err(anyhow!("Error").into())
    }
}