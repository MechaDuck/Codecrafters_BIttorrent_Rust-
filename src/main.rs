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
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: <command> [args]");
        return;
    }
    let command = &args[1];
    let mut torrent_manager = TorrentManager::new(&encode_bencoded_value, &decode_bencoded_value);

    match command.as_str() {
        "decode" => decode_command(&args),
        "info" => info_command(&mut torrent_manager, &args),
        "peers" => peers_command(&mut torrent_manager, &args),
        "handshake" => handshake_command(&mut torrent_manager, &args).await,
        "download_piece" => download_piece_command(&mut torrent_manager, &args).await,
        "download" => download_command(&mut torrent_manager, &args).await,
        _ => println!("unknown command: {}", command),
    }
}

// Decode a bencoded value passed as an argument
fn decode_command(args: &[String]) {
    if args.len() < 3 {
        println!("Usage: decode <encoded_value>");
        return;
    }
    let encoded_value = &args[2];
    let decoded_value = decode_bencoded_value(encoded_value.as_bytes(), true);
    match decoded_value {
        Ok((value, _)) => println!("{}", value.to_string()),
        Err(e) => println!("Failed to decode: {}", e),
    }
}

// Print meta information of a torrent file
fn info_command(torrent_manager: &mut TorrentManager, args: &[String]) {
    if args.len() < 3 {
        println!("Usage: info <file>");
        return;
    }
    let file = &args[2];
    let content = filereader::read_file_as_vector(file).unwrap();
    let _ = torrent_manager.parse_meta_info_file(content);
    let _ = torrent_manager.print_meta_info();
}

// Print peers of a torrent file
fn peers_command(torrent_manager: &mut TorrentManager, args: &[String]) {
    if args.len() < 3 {
        println!("Usage: peers <file>");
        return;
    }
    let file = &args[2];
    let content = filereader::read_file_as_vector(file).unwrap();
    let _ = torrent_manager.parse_meta_info_file(content);
    let _ = torrent_manager.init_clients();
    let _ = torrent_manager.print_peers();
}

// Perform a handshake with a peer
async fn handshake_command(torrent_manager: &mut TorrentManager<'_>, args: &[String]) {
    if args.len() < 4 {
        println!("Usage: handshake <file> <peer_address>");
        return;
    }
    let file = &args[2];
    let peer_address = &args[3];
    let content = filereader::read_file_as_vector(file).unwrap();
    let _ = torrent_manager.parse_meta_info_file(content);
    let _ = torrent_manager.init_clients();
    match torrent_manager.perform_peer_handshake(peer_address).await {
        Ok(resp) => print!("Peer ID: {}\n", hex::encode(resp[48..].to_vec())),
        Err(e) => println!("Handshake failed: {}", e),
    }
}

// Download a specific piece from a torrent file
async fn download_piece_command(torrent_manager: &mut TorrentManager<'_>, args: &[String]) {
    if args.len() < 6 {
        println!("Usage: download_piece <file> <output_path> <piece_index>");
        return;
    }
    let output_path = &args[3];
    let meta_file = &args[4];
    let piece_index = &args[5];

    let content = filereader::read_file_as_vector(meta_file).unwrap();
    let _ = torrent_manager.parse_meta_info_file(content);
    let _ = torrent_manager.init_clients();
    match torrent_manager.download_piece_with_index(piece_index.parse::<u32>().unwrap()).await {
        Ok(piece) => {
            let _ = filereader::write_vector_to_file(output_path, piece);
            println!("Piece {} downloaded to {}", piece_index, output_path);
        }
        Err(e) => println!("Failed to download piece: {}", e),
    }
}

// Download the entire file from a torrent
async fn download_command(torrent_manager: &mut TorrentManager<'_>, args: &[String]) {
    if args.len() < 5 {
        println!("Usage: download <file> <output_path>");
        return;
    }
    let output_path = &args[3];
    let file = &args[4];
    let content = filereader::read_file_as_vector(file).unwrap();
    let _ = torrent_manager.parse_meta_info_file(content);
    let _ = torrent_manager.init_clients();
    match torrent_manager.download_file().await {
        Ok(file) => {
            let _ = filereader::write_vector_to_file(output_path, file);
            println!("File downloaded to {}", output_path);
        }
        Err(e) => println!("Failed to download file: {}", e),
    }
}
