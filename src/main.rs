mod bencoded_parser;
mod filereader;
mod metainfo;

use std::env;

// Main function to handle command-line arguments and execute commands
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = bencoded_parser::decode_bencoded_value(encoded_value.as_bytes());
        println!("{}", decoded_value.unwrap().0.to_string());
    } else if command == "info" {
        let file = &args[2];
        print_metainfo(file);

    } else {
        println!("unknown command: {}", args[1])
    }
}


fn print_metainfo(file: &String) {
    let content = filereader::read_file_as_vector(file).unwrap();
    let decoded_value = bencoded_parser::decode_bencoded_value(&content).unwrap().0;
    let tracker_url = decoded_value["announce"].as_str().unwrap().to_string();
    let length = decoded_value["info"]["length"].as_i64().unwrap();
    let metainfo = metainfo::Metainfo::new(tracker_url, length);
    print!("{}", metainfo.get_formatted_info());
}