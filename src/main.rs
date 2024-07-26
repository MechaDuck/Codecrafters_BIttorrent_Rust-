mod clients;
mod file_processing;
mod bencode_processing;
mod utils;
mod torrent_manager;

use base64::{Engine as _, engine::general_purpose};
use anyhow::{anyhow, Ok, Result};
use clients::tracker_client;
use file_processing::filereader;
use serde_json::Value;
use torrent_manager::torrent_manager::TorrentManager;
use std::env;
use std::net::Ipv4Addr;

use bencode_processing::decoder::decode_bencoded_value;
use bencode_processing::encoder::encode_bencoded_value;
use tracker_client::TrackerClient;


// Main function to handle command-line arguments and execute commands
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    let mut torrent_manager = TorrentManager::new(&encode_bencoded_value, &decode_bencoded_value);

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value.as_bytes(), true);
        println!("{}", decoded_value.unwrap().0.to_string());

        
    } else if command == "info" {
        let file = &args[2];
        let content = filereader::read_file_as_vector(file).unwrap();
        let _ = torrent_manager.parse_meta_info_file(content);
        let _ = torrent_manager.print_meta_info();

    } else if command == "peers" {
        let file = &args[2];
        let decoded_values = read_metainfo(file);
        let tracker_url = utils::decode_base64_to_utf8_string(decoded_values["announce"].as_str().unwrap()).unwrap();

        let tracker_client = TrackerClient::new(tracker_url);

        let info_data = &decoded_values["info"];
        let encoded_info = encode_bencoded_value(info_data).unwrap();
        let hex_info_hash = utils::calculate_sha1_hash(encoded_info);

        let length = decoded_values["info"]["length"].as_i64().unwrap();

        let resp = tracker_client.request_peers(length, hex_info_hash).unwrap();
        let decoded_value = decode_bencoded_value(&resp, false).unwrap().0;

        let extracted_peers = extract_peers(decoded_value["peers"].as_str().unwrap().to_string()).unwrap();

        print_peers(extracted_peers)
        
    } else {
        println!("unknown command: {}", args[1])
    }
}

fn read_metainfo(file: &String) -> Value {
    let content = filereader::read_file_as_vector(file).unwrap();
    let decoded_value = decode_bencoded_value(&content, false).unwrap().0;
    return decoded_value;
}


fn extract_peers(peers_base64: String) -> Result<Vec<String>> {
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

fn print_peers(peers: Vec<String>) {
    for peer in peers{
        print!("{}\n", peer);
    }
    
}