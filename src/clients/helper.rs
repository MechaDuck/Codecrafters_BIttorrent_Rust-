use std::collections::HashMap;

pub fn create_request_url(root_url: String, params: HashMap<&str, String>) -> String {
    let mut query_string = String::new();
    for (key, value) in &params {
        if !query_string.is_empty() {
            query_string.push('&');
        }
        query_string.push_str(&format!("{}={}", key, value));
    }
    
    // Construct the full URL
    format!("{}?{}", root_url, query_string)
}

