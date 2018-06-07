extern crate rson;
use rson::*;
use std::path::Path;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let filename = Path::new("mappings.json");
    let response_to_slack = "{\"text\": \"✅ Your request has been received!\"}";
    match start_server(addr, filename, response_to_slack) {
        Ok(_) => {}
        Err(error) => panic!(error),
    }
}
