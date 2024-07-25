
pub struct Metainfo {
    tracker_url: String,
    length: i64,
    hash: String,
    piece_length: i64,
    piece_hashes: String,
}

impl Default for Metainfo {
    fn default() -> Self {
        Self {
            tracker_url: String::new(),
            length: 0,
            hash: String::new(),
            piece_length: 0,
            piece_hashes: String::new(),
        }
    }
}

impl Metainfo {
    // Create a new Metainfo with a specified status line
    pub fn new(tracker_url: String, length: i64, hash: String, piece_length: i64, piece_hashes: String) -> Self {
        Self { tracker_url, length, hash, piece_length,  piece_hashes}
    }

    // return info string
    pub fn get_formatted_info(&self) -> String{
        let mut info = format!("Tracker URL: {} \n", self.tracker_url);
        info.push_str(format!("Length: {} \n", self.length).as_str());
        info.push_str(format!("Info Hash: {} \n", self.hash).as_str());
        info.push_str(format!("Piece Length: {} \n", self.piece_length).as_str());
        info.push_str(format!("Piece Hashes: {} \n", self.piece_hashes).as_str());
        info

    }

    
}
