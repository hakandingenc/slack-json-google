#![allow(dead_code, unused_imports)]
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate url;

// use futures::{Future, Stream};
// use hyper::Client;
// use hyper::header::{ContentLength, ContentType};
// use hyper::server::{Http, Response, Service};
// use hyper::{Method, Request, StatusCode};
// use hyper_tls::HttpsConnector;
// use std::collections::HashMap;
// use std::io::{self, Write};
// use tokio_core::reactor::Core;
// use url::{form_urlencoded, Host, ParseError, Url};

use futures::{Stream, future::Future};
use hyper::{Body, Chunk, Method, StatusCode, header::ContentLength,
            server::{Http, Request, Response, Service}};
use std::collections::HashMap;
use url::form_urlencoded;

const GET_RESPONSE: &'static str = "This server expects POST requests to /";
static MISSING: &[u8] = b"Missing field";
const NUM_THREADS: usize = 4;

pub struct Forwarder {
    payload: String,
    webhook: hyper::Uri,
}

pub struct SimpleRespond;
//  {
//     JSONpayload: String,
// }

impl Service for SimpleRespond {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    //type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
    type Response = Response;

    fn call(&self, req: Request) -> Self::Future {
        let mut response = Response::new();

        match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                response.set_body("This server expects POST requests to /");
            }
            (&Method::Post, "/") => {
                return Box::new(req.body().concat2().map(|b| {
                    let params = form_urlencoded::parse(b.as_ref())
                        .into_owned()
                        .collect::<HashMap<String, String>>();
                    //println!("{:?}", params);
                    let res_url = if let Some(n) = params.get("payload") {
                        n
                    } else {
                        return Response::new()
                            .with_status(StatusCode::UnprocessableEntity)
                            .with_header(ContentLength(MISSING.len() as u64))
                            .with_body(MISSING);
                    };
                    let body = format!("Hello {}, your url is", res_url);
                    Response::new()
                        .with_header(ContentLength(body.len() as u64))
                        .with_body(body)
                }));

                //println!("{:?}", req.body());
                //response.set_body(req.body());
                //response.set_status(StatusCode::Ok);
            }
            _ => {
                response.set_status(StatusCode::NotFound);
            }
        };

        Box::new(futures::future::ok(response))
    }
}

fn to_uppercase(chunk: Chunk) -> Chunk {
    let uppered = chunk
        .iter()
        .map(|byte| byte.to_ascii_uppercase())
        .collect::<Vec<u8>>();
    Chunk::from(uppered)
}

// impl Forwarder {
//     pub fn new(payload: String, webhook: &str) -> Self {
//         Forwarder {
//             payload,
//             webhook: webhook.parse().unwrap(),
//         }
//     }
//
//     pub fn send(&self) {
//         let mut core = Core::new().unwrap();
//         let handle = core.handle();
//         let client = Client::configure()
//             .connector(HttpsConnector::new(NUM_THREADS, &handle).unwrap())
//             .build(&handle);
//         let mut req = Request::new(Method::Post, self.webhook.clone());
//         req.headers_mut().set(ContentType::json());
//         req.headers_mut()
//             .set(ContentLength(self.payload.len() as u64));
//         req.set_body(self.payload.clone());
//         println!("{:?}", req);
//         let post = client.request(req).and_then(|res| {
//             println!("POST: {:?}", res);
//
//             res.body().concat2()
//         });
//
//         core.run(post).unwrap();
//     }
// }

struct ForwardingClient {}
