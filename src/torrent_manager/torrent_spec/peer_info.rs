pub struct Peer {
    ip_address: String, // format <ip_address:port> e.g. 123.123.123.123:1234
}


impl Peer {
    pub fn new(ip_address: String) -> Self {
        Self{ip_address}
    }

    pub fn get_ip_address(&self) -> &String {
        &self.ip_address
    }
}