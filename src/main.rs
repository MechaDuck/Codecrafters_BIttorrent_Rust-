mod clients;
mod file_processing;
mod bencode_processing;
mod utils;
mod torrent_manager;

use file_processing::filereader;
use torrent_manager::torrent_manager::TorrentManager;
use std::env;

use bencode_processing::decoder::decode_bencoded_value;
use bencode_processing::encoder::encode_bencoded_value;



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
        let content = filereader::read_file_as_vector(file).unwrap();
        let _ = torrent_manager.parse_meta_info_file(content);
        let _ = torrent_manager.print_peers();
        
    } else {
        println!("unknown command: {}", args[1])
    }
}



