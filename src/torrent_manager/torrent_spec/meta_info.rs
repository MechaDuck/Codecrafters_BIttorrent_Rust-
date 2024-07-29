pub struct Metainfo {
    tracker_url: Option<String>,
    length: Option<i64>,
    hash: Option<String>,
    piece_length: Option<i64>,
    piece_hashes: Option<String>,
}

impl Default for Metainfo {
    fn default() -> Self {
        Self {
            tracker_url: None,
            length: None,
            hash: None,
            piece_length: None,
            piece_hashes: None,
        }
    }
}

impl Metainfo {
    // Create a new Metainfo with a specified status line
    pub fn new() -> Self {
        Self::default()
    }
    // Setter for tracker_url
    pub fn set_tracker_url(&mut self, url: String) {
        self.tracker_url = Some(url);
    }

    // Getter for tracker_url
    pub fn get_tracker_url(&self) -> &Option<String>{
        &self.tracker_url
    }

    // Setter for length
    pub fn set_length(&mut self, length: i64) {
        self.length = Some(length);
    }

    // Getter for length
    pub fn get_length(&self) -> &Option<i64>{
        &self.length
    }


    // Setter for hash
    pub fn set_hash(&mut self, hash: String) {
        self.hash = Some(hash);
    }

    // Getter for tracker_url
    pub fn get_hash(&self) -> &Option<String>{
        &self.hash
    }


    // Setter for piece_length
    pub fn set_piece_length(&mut self, piece_length: i64) {
        self.piece_length = Some(piece_length);
    }

    // Getter for piece_length
    pub fn get_piece_length(&self) -> &Option<i64>{
        &self.piece_length
    }

    // Setter for piece_hashes
    pub fn set_piece_hashes(&mut self, piece_hashes: String) {
        self.piece_hashes = Some(piece_hashes);
    }

    // Getter for piece_hashes
    pub fn get_piece_hashes(&self) -> &Option<String>{
        &self.piece_hashes
    }


    // return info string
    pub fn get_formatted_info(&self) -> String{
        let tracker_url = self.tracker_url.as_ref().map_or("N/A", |url| url.as_str());
        let length = self.length.map_or("N/A".to_string(), |l| l.to_string());
        let hash = self.hash.as_ref().map_or("N/A", |h| h.as_str());
        let piece_length = self.piece_length.map_or("N/A".to_string(), |pl| pl.to_string());
        let piece_hashes = self.piece_hashes.as_ref().map_or("N/A", |ph| ph.as_str());

        format!(
            "Tracker URL: {}\nLength: {}\nInfo Hash: {}\nPiece Length: {}\nPiece Hashes: {}\n",
            tracker_url,
            length,
            hash,
            piece_length,
            piece_hashes
        )

    }

    
}
