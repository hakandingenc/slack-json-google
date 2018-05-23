#[ignore(dead_code)]
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate url;

use futures::{Future, Stream};
use hyper::Client;
use hyper::header::{ContentLength, ContentType};
use hyper::{Method, Request};
use hyper_tls::HttpsConnector;
use std::io::{self, Write};
use tokio_core::reactor::Core;
use url::{Host, ParseError, Url};

pub struct Forwarder {
    payload: String,
    webhook: hyper::Uri,
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
            .connector(HttpsConnector::new(4, &handle).unwrap())
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
