pub struct Peer {
    ip_address: String, // format <ip_address:port> e.g. 123.123.123.123:1234
}

impl Default for Peer {
    fn default() -> Self {
        Self {
            ip_address: String::new(),
        }
    }
}

impl Peer {
    
}