#![allow(dead_code, unused_imports)]
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde_json;
extern crate tokio_core;
extern crate url;

use futures::{Stream, future::Future};
use hyper::{Body, Chunk, Error, Method, StatusCode, header::ContentLength,
            server::{Http, Request, Response, Service}};
use serde_json::{Error as SerdeError, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use url::form_urlencoded;

// For deriving serde macros
extern crate serde;
#[macro_use]
extern crate serde_derive;

// For Client
//use futures::Future;
use hyper::Client;
//use hyper::Request;
use hyper::header::ContentType;
use std::io::{self, Write};
use tokio_core::reactor::Core;

const GET_RESPONSE: &'static str = "This server expects POST requests to /";
static MISSING: &[u8] = b"Missing field";
const NUM_THREADS: usize = 4;

pub struct SimpleRespond(pub tokio_core::reactor::Handle);

pub struct Mappings {
    transform: HashMap<String, String>,
}
// For extra client
pub type ResponseStream = Box<Stream<Item = Chunk, Error = Error>>;

impl Service for SimpleRespond {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    //type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
    // For extra client
    //Before: type Response = Response;
    type Response = Response<ResponseStream>;

    fn call(&self, req: Request) -> Self::Future {
        let mut response = Response::new();

        match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                let body: ResponseStream = Box::new(hyper::Body::from(GET_RESPONSE));
                response.set_body(body);
            }
            (&Method::Post, "/") => {
                let handle = self.0.clone();
                return Box::new(req.body().concat2().map(move |b| {
                    let params = form_urlencoded::parse(b.as_ref())
                        .into_owned()
                        .collect::<HashMap<String, String>>();
                    let res_url: Value = if let Some(n) = params.get("payload") {
                        serde_json::from_str(n).unwrap()
                    } else {
                        let body: ResponseStream = Box::new(hyper::Body::from(GET_RESPONSE));

                        return Response::new()
                            .with_status(StatusCode::UnprocessableEntity)
                            .with_header(ContentLength(MISSING.len() as u64))
                            .with_body(body);
                    };
                    // Use the client in some way
                    //let client = Client::new(&handle);
                    let client = ::hyper::Client::configure()
                        .connector(::hyper_tls::HttpsConnector::new(4, &handle).unwrap())
                        .build(&handle);
                    //let client = Client::configure().build(&self.0);
                    //let mut req2 = Request::new(Method::Get, "localhost/".parse().unwrap());
                    //req2.set_body("ABC");
                    //let uri = "http://httpbin.org/ip".parse().unwrap();
                    let uri = "https://script.google.com/macros/s/AKfycbzqs6D4QA8L2x2k9B3_UrgSU1Vcqj0icHiIs26G0IbTYaBNy8xW/exec".parse().unwrap();
                    let mut request = Request::new(Method::Post, uri);
                    request.set_body(Body::from("payload=Hello%20world"));
                    {
                        let mut headers = request.headers_mut();
                        headers.set_raw("Content-Type", "application/x-www-form-urlencoded");
                        headers.set_raw("Accept", "*/*");
                        headers.set_raw("User-Agent", "Rust");
                    }
                    let work = client.request(request).and_then(|res| {
                        println!("Response: {}", res.status());

                        res.body()
                            .for_each(|chunk| io::stdout().write_all(&chunk).map_err(From::from))
                    });
                    &handle.spawn(work.map_err(|_| ()));

                    // Continue with the server
                    let body = format!(
                        "The mapping for {} is {}\n",
                        &res_url["callback_id"],
                        resolve_callback(&res_url["callback_id"])
                    );
                    let len = body.len();
                    let body: ResponseStream = Box::new(hyper::Body::from(body));
                    Response::new()
                        .with_header(ContentLength(len as u64))
                        .with_body(body)
                }));
            }
            _ => {
                response.set_status(StatusCode::NotFound);
            }
        };

        Box::new(futures::future::ok(response))
    }
}

fn resolve_callback(id: &serde_json::Value) -> serde_json::Value {
    let mut f = File::open("mappings.json").expect("file not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    let json: Value = serde_json::from_str(&contents).unwrap();
    json[id.as_str().unwrap()].clone()
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dictionary {
    mappings: HashMap<String, String>,
}

impl Dictionary {
    pub fn new_from_file(path: &str) -> io::Result<Self> {
        let mut f = File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let mappings: HashMap<String, String> = serde_json::from_str(contents.as_ref())?;
        println!("{:?}", mappings);
        Ok(Dictionary { mappings })
    }
    pub fn resolve_callback(&self, id: &str) -> Option<&String> {
        self.mappings.get(id)
    }
}
