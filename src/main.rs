extern crate rson;
use rson::*;
use std::path::Path;
pub mod test;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let filename = Path::new("mappings.json");
    start_server(addr, filename);
}
