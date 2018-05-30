// extern crate futures;
// extern crate hyper;
// extern crate rson;
// extern crate tokio_core;
// use hyper::server::Http;
// use rson::*;

extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
extern crate tokio_core;

use futures::{Future, Stream};

use hyper::error::Error;
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Body, Chunk, Client, Get, Post, StatusCode};
extern crate rson;
use rson::*;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let server_handle = core.handle();
    let client_handle = core.handle();
    let serve = Http::new()
        .serve_addr_handle(&addr, &server_handle, move || {
            Ok(SimpleRespond(client_handle.clone()))
        })
        .unwrap();
    println!(
        "Listening on http://{} with 1 thread.",
        serve.incoming_ref().local_addr()
    );
    let h2 = server_handle.clone();
    server_handle.spawn(
        serve
            .for_each(move |conn| {
                h2.spawn(
                    conn.map(|_| ())
                        .map_err(|err| println!("serve error: {:?}", err)),
                );
                Ok(())
            })
            .map_err(|_| ()),
    );
    core.run(futures::future::empty::<(), ()>()).unwrap();
}
