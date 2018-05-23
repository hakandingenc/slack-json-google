extern crate hyper;
extern crate rson;
use hyper::server::Http;
use rson::*;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(Receiver)).unwrap();
    server.run().unwrap();
}
