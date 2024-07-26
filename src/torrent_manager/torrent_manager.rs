use crate::utils;
use super::torrent_spec::{self, meta_info::Metainfo};
use serde_json::Value;


use anyhow::{anyhow, Ok, Result};

// Define a function type for encoding
type EncoderFn = dyn Fn(&Value) -> Result<Vec<u8>>;

// Define a function type for decoding
type DecoderFn = dyn Fn(&[u8], bool) -> Result<(Value, &[u8])>;

pub struct TorrentManager<'a> {
    encoder: &'a EncoderFn,
    decoder: &'a DecoderFn,
    metainfo: Option<torrent_spec::meta_info::Metainfo>,
}


impl<'a> TorrentManager<'a> {
    pub fn new(encoder: &'a EncoderFn, decoder: &'a DecoderFn) -> Self {
        
        TorrentManager { encoder, decoder, metainfo: None }
    }

    pub fn parse_meta_info_file(&mut self, data: Vec<u8>) -> Result<()>{
        let decoded_value = (self.decoder)(&data, false)?.0;
        let mut metainfo = torrent_spec::meta_info::Metainfo::new();

        metainfo.set_tracker_url(utils::decode_base64_to_utf8_string(decoded_value["announce"].as_str().unwrap()).unwrap());
        metainfo.set_length(decoded_value["info"]["length"].as_i64().unwrap());
        metainfo.set_piece_length(decoded_value["info"]["piece length"].as_i64().unwrap());
        metainfo.set_piece_hashes( utils::decode_base64_to_hex(decoded_value["info"]["pieces"].as_str().unwrap())?);
        
        let info_data = &decoded_value["info"];
        let encoded_info = (self.encoder)(info_data)?;
        let hash = utils::calculate_sha1_hash(encoded_info);
        metainfo.set_hash(hash);

        self.metainfo = Some(metainfo);
        Ok(())

    }

    pub fn print_meta_info(&self) -> Result<()>{
        match &self.metainfo {
            Some(metainfo) => {
                print!("{}", metainfo.get_formatted_info());
            },
            None => {
                return Err(anyhow!("Error: meta info was not initialized!"));

            }
        }
        Ok(())
    }

}