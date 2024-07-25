pub struct Metainfo {
    tracker_url: String,
    length: i64,
}

impl Default for Metainfo {
    fn default() -> Self {
        Self {
            tracker_url: String::new(),
            length: 0,
        }
    }
}

impl Metainfo {
    // Create a new Metainfo with a specified status line
    pub fn new(tracker_url: String, length: i64) -> Self {
        Self { tracker_url, length }
    }

    pub fn get_formatted_info(&self) -> String{
        
        let mut info = format!("Tracker URL: {} \n", self.tracker_url);
        info.push_str(format!("Length: {} \n", self.length).as_str());
        info

    }

}
