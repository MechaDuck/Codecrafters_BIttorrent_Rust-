use reqwest::Client;


pub struct PeerClient {
    client: Client,
    root_url: String,
}


impl Default for PeerClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
            root_url: "".to_string(),
        }
    }
}

impl PeerClient {

    pub fn new(root_url: String) -> Self {
        Self{root_url, ..Default::default()}
    }
    
    pub fn perform_handshake(&self) {


    } 
}