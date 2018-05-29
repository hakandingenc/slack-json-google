#![allow(dead_code, unused_imports)]
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde_json;
extern crate tokio_core;
extern crate url;

use futures::{Stream, future::Future};
use hyper::{Body, Chunk, Method, StatusCode, header::ContentLength,
            server::{Http, Request, Response, Service}};
use serde_json::{Error, Value};
use std::collections::HashMap;
use url::form_urlencoded;

const GET_RESPONSE: &'static str = "This server expects POST requests to /";
static MISSING: &[u8] = b"Missing field";
const NUM_THREADS: usize = 4;

pub struct SimpleRespond;

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
                response.set_body(GET_RESPONSE);
            }
            (&Method::Post, "/") => {
                return Box::new(req.body().concat2().map(|b| {
                    let params = form_urlencoded::parse(b.as_ref())
                        .into_owned()
                        .collect::<HashMap<String, String>>();
                    let res_url: Value = if let Some(n) = params.get("payload") {
                        serde_json::from_str(n).unwrap()
                    } else {
                        return Response::new()
                            .with_status(StatusCode::UnprocessableEntity)
                            .with_header(ContentLength(MISSING.len() as u64))
                            .with_body(MISSING);
                    };
                    let body = format!("The callback_id", res_url["callback_id"]);
                    Response::new()
                        .with_header(ContentLength(body.len() as u64))
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

fn load_file() {}
