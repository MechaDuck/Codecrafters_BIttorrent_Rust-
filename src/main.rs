mod bencode_decoder;
mod filereader;
mod metainfo;
mod bencode_encoder;
mod tracker_client;
use base64::{Engine as _, engine::general_purpose};
use anyhow::{anyhow, Ok, Result};
use serde_json::Value;
use sha1::{Sha1, Digest};

use std::env;

// Main function to handle command-line arguments and execute commands
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = bencode_decoder::decode_bencoded_value(encoded_value.as_bytes(), true);
        println!("{}", decoded_value.unwrap().0.to_string());
    } else if command == "info" {
        let file = &args[2];
        print_metainfo(file);
    } else if command == "peers" {
        let file = &args[2];
        let decoded_values = read_metainfo(file);
        let tracker_url = decode_base64_to_utf8_string(decoded_values["announce"].as_str().unwrap()).unwrap();

        let tracker_client = tracker_client::TrackerClient::new(tracker_url);
        let hex_info_hash = calculate_hash_of_info(&decoded_values["info"]).unwrap();
        let length = decoded_values["info"]["length"].as_i64().unwrap();

        let resp = tracker_client.request_peers(length, hex_info_hash).unwrap();
        let decoded_value = bencode_decoder::decode_bencoded_value(&resp, false).unwrap().0;
        print!("{}", decoded_value["peers"].as_str().unwrap().to_string());
        
    } else {
        println!("unknown command: {}", args[1])
    }
}

fn read_metainfo(file: &String) -> Value {
    let content = filereader::read_file_as_vector(file).unwrap();
    let decoded_value = bencode_decoder::decode_bencoded_value(&content, false).unwrap().0;
    return decoded_value;
}
fn print_metainfo(file: &String) {
    let content = filereader::read_file_as_vector(file).unwrap();
    let decoded_value = bencode_decoder::decode_bencoded_value(&content, false).unwrap().0;
  
    let tracker_url = decode_base64_to_utf8_string(decoded_value["announce"].as_str().unwrap()).unwrap();
    let length = decoded_value["info"]["length"].as_i64().unwrap();

    let hash = calculate_hash_of_info(&decoded_value["info"]).unwrap();
    let piece_length = decoded_value["info"]["piece length"].as_i64().unwrap();
    let pieces = decode_base64_to_hex(decoded_value["info"]["pieces"].as_str().unwrap()).unwrap();
    
    // TODO: Implement pieces and piece length
    let metainfo = metainfo::Metainfo::new(tracker_url, length, hash, piece_length, pieces);
    print!("{}", metainfo.get_formatted_info());
}

fn decode_base64_to_utf8_string(base64_string: &str) -> Result<String> {
    let bytes_string = general_purpose::STANDARD.decode(base64_string).map_err(|e| anyhow!(e))?;
    let utf8_string = std::str::from_utf8(&bytes_string).map_err(|e| anyhow!(e))?;
    Ok(utf8_string.to_string())
}

fn decode_base64_to_hex(base64_string: &str) -> Result<String> {
    let bytes_string = general_purpose::STANDARD.decode(base64_string).map_err(|e| anyhow!(e))?;
    let hex_string = hex::encode(bytes_string); 
    Ok(hex_string)
}



fn calculate_hash_of_info(info_dict: &Value) -> Result<String> {
    let encoded_info = bencode_encoder::encode_value(info_dict)?;
    let mut hasher = Sha1::new();
    hasher.update(encoded_info);
    let result = hasher.finalize();
    let hex_string = hex::encode(result);
    return Ok(hex_string);
}