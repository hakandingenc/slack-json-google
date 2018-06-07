extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate url;

use futures::{Stream, future::Future};
use hyper::{Body, Chunk, Error, Method, StatusCode, header::ContentLength,
            server::{Request, Response, Service}};
use serde_json::Value;
use std::{collections::HashMap, io::{self, prelude::*},
          sync::{Arc, Mutex, mpsc::{Receiver, Sender}}};
use url::form_urlencoded;

const GET_RESPONSE: &str = "This server expects POST requests";
const MISSING: &str = "Missing field";
const PAYLOAD: &str = "payload";
const NUM_THREADS: usize = 4;

pub struct Server {
    core_handle: tokio_core::reactor::Handle,
    response_to_slack: String,
    send_callback_id: Sender<String>,
    recv_url: Arc<Mutex<Receiver<Option<String>>>>,
}

type ResponseStream = Box<Stream<Item = Chunk, Error = Error>>;

impl Server {
    pub fn new(
        core_handle: tokio_core::reactor::Handle,
        response_to_slack: String,
        send_callback_id: Sender<String>,
        recv_url: Arc<Mutex<Receiver<Option<String>>>>,
    ) -> Self {
        Server {
            core_handle,
            response_to_slack,
            send_callback_id,
            recv_url,
        }
    }
}

impl Service for Server {
    type Request = Request;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
    type Response = Response<ResponseStream>;

    fn call(&self, req: Request) -> Self::Future {
        let mut response = Response::new();

        match req.method() {
            &Method::Post => {
                let handle = self.core_handle.clone();
                let response_to_slack = self.response_to_slack.clone();
                let send_callback_id = self.send_callback_id.clone();
                let recv_url = self.recv_url.clone();

                return Box::new(req.body().concat2().map(move |b| {
                    let parameter = form_urlencoded::parse(b.as_ref())
                        .into_owned()
                        .collect::<HashMap<String, String>>();

                    let url_encoded_payload = parameter.get(PAYLOAD);

                    let parsed_payload: Value = if let Some(n) = url_encoded_payload {
                        serde_json::from_str(n).unwrap()
                    } else {
                        let body: ResponseStream = Box::new(hyper::Body::from(GET_RESPONSE));

                        return Response::new()
                            .with_status(StatusCode::UnprocessableEntity)
                            .with_header(ContentLength(MISSING.len() as u64))
                            .with_body(body);
                    };
                    let client = ::hyper::Client::configure()
                        .connector(::hyper_tls::HttpsConnector::new(NUM_THREADS, &handle).unwrap())
                        .build(&handle);

                    send_callback_id
                        .send(parsed_payload["callback_id"].as_str().unwrap().to_string());
                    let uri = recv_url
                        .lock()
                        .unwrap()
                        .recv()
                        .unwrap()
                        .unwrap()
                        .parse()
                        .unwrap();
                    let mut request = Request::new(Method::Post, uri);
                    request.set_body(Body::from(b));
                    {
                        let headers = request.headers_mut();
                        headers.set_raw("Content-Type", "application/x-www-form-urlencoded");
                    }
                    let work = client.request(request).and_then(|res| {
                        println!("Response: {}", res.status());

                        res.body()
                            .for_each(|chunk| io::stdout().write_all(&chunk).map_err(From::from))
                    });
                    &handle.spawn(work.map_err(|_| ()));

                    // Continue with the server
                    let body = response_to_slack;
                    let len = body.len();
                    let body: ResponseStream = Box::new(hyper::Body::from(body));
                    Response::new()
                        .with_header(ContentLength(len as u64))
                        .with_body(body)
                }));
            }
            &Method::Get => {
                let body: ResponseStream = Box::new(hyper::Body::from(GET_RESPONSE));
                response.set_body(body);
            }
            _ => {
                response.set_status(StatusCode::NotFound);
            }
        };

        Box::new(futures::future::ok(response))
    }
}
