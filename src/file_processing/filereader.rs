use std::{fs::File, io::{Read, Write}};
use anyhow::Error;

// Read file contents into a Vec<u8>
pub fn read_file_as_vector(filename: &str) -> Result<Vec<u8>, Error> {
    let mut file = File::open(filename)?;
    let mut content = vec![];
    file.read_to_end(&mut content)?;
    Ok(content)
}


pub fn write_vector_to_file(filename: &str, content: Vec<u8>) -> Result<(), Error> {
    let mut file = File::create(filename)?;
    file.write_all(&content)?;
    Ok(())
}