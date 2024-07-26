use std::{fs::File, io::Read};
use anyhow::Error;

// Read file contents into a Vec<u8>
pub fn read_file_as_vector(filename: &str) -> Result<Vec<u8>, Error> {
    let mut file = File::open(filename)?;
    let mut content = vec![];
    file.read_to_end(&mut content)?;
    Ok(content)
}
