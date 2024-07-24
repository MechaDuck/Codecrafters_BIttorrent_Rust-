mod bencoded_parser;
mod filereader;
use serde_json;

use std::env;

// Main function to handle command-line arguments and execute commands
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = bencoded_parser::decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.unwrap().0.to_string());
    } else if command == "info" {
        let filename = &args[2];
        println!("Missing implementation!");
        // let content = filereader::read_file_as_vector(filename).unwrap();
        // let decoded_value = bencoded_parser::decode_bencoded_value(content.as);
        // println!("{}", decoded_value.unwrap().0.to_string());

    } else {
        println!("unknown command: {}", args[1])
    }
}

