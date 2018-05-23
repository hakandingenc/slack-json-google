#[ignore(dead_code)]
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate url;

use futures::{Future, Stream};
use hyper::Client;
use hyper::header::{ContentLength, ContentType};
use hyper::server::{Http, Response, Service};
use hyper::{Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use std::collections::HashMap;
use std::io::{self, Write};
use tokio_core::reactor::Core;
use url::{form_urlencoded, Host, ParseError, Url};

const NUM_THREADS: usize = 4;

pub struct Forwarder {
    payload: String,
    webhook: hyper::Uri,
}

pub struct Receiver;

impl Service for Receiver {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let mut response = Response::new();

        match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                response.set_body("This server expects POST requests to /");
            }
            (&Method::Post, "/") => {
                Box::new(req.body().concat2().map(|b| {
                    let my_string = String::from_utf8_lossy(b.as_ref());
                    println!("{:?}", my_string);

                    let body = format!("Hello {}, your number is", my_string);
                    // return Box::new(futures::future::ok(
                    //     Response::new()
                    //         .with_header(ContentLength(body.len() as u64))
                    //         .with_body(body),
                    // ));
                }));
                println!("ABC");
                // let my_forwarder = Forwarder::new(
                //     my_string,
                //     "https://hooks.slack.com/services/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",
                // );
                // my_forwarder.send();
            }
            _ => {
                response.set_status(StatusCode::NotFound);
            }
        };

        Box::new(futures::future::ok(response))
    }
}

impl Forwarder {
    pub fn new(payload: String, webhook: &str) -> Self {
        Forwarder {
            payload,
            webhook: webhook.parse().unwrap(),
        }
    }

    pub fn send(&self) {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(NUM_THREADS, &handle).unwrap())
            .build(&handle);
        let mut req = Request::new(Method::Post, self.webhook.clone());
        req.headers_mut().set(ContentType::json());
        req.headers_mut()
            .set(ContentLength(self.payload.len() as u64));
        req.set_body(self.payload.clone());
        println!("{:?}", req);
        let post = client.request(req).and_then(|res| {
            println!("POST: {:?}", res);

            res.body().concat2()
        });

        core.run(post).unwrap();
    }
}

struct Forwarding_Client {}
