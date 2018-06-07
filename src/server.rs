extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate url;
extern crate mime;

use futures::{Stream, future::Future};
use hyper::{Body, Chunk, Error, Method, StatusCode, header::{ContentLength,ContentType},
            server::{Request, Response, Service}};
use serde_json::Value;
use std::{collections::HashMap, io::{self, prelude::*},
          sync::{Arc, Mutex, mpsc::{Receiver, Sender}}};
use url::form_urlencoded;

const GET_RESPONSE: &str = "This server expects POST requests";
const PAYLOAD: &str = "payload";
const CALLBACK_ID : &str = "callback_id";
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

                return Box::new(req.body().concat2().map(move |assembled_body| {
                    let parameter = form_urlencoded::parse(assembled_body.as_ref())
                        .into_owned()
                        .collect::<HashMap<String, String>>();

                    let url_encoded_payload = parameter.get(PAYLOAD);

                    let parsed_payload: Value = if let Some(n) = url_encoded_payload {
                        serde_json::from_str(n).expect("Couldn't parse formurl into JSON")
                    } else {
                        let body: ResponseStream = Box::new(hyper::Body::from(GET_RESPONSE));

                        return Response::new()
                            .with_status(StatusCode::UnprocessableEntity)
                            .with_header(ContentLength(GET_RESPONSE.len() as u64))
                            .with_body(body);
                    };
                    let client = ::hyper::Client::configure()
                        .connector(::hyper_tls::HttpsConnector::new(NUM_THREADS, &handle).unwrap())
                        .build(&handle);

                    send_callback_id
                        .send(parsed_payload[CALLBACK_ID].as_str().unwrap().to_string());

                    let uri = recv_url
                        .lock()
                        .expect("Mutex poisoned")
                        .recv()
                        .expect("Error communicating with hostmap thread")
                        .expect("Requested callback_id has no mapping") //This should be a match, crashing b/c of nonexistent ID isn't optimal
                        .parse()
                        .expect("Could not parse mapping into URL"); //This should also be a match, crashing server from bad URL isn't optimal

                    let mut request = Request::new(Method::Post, uri);
                    request.set_body(Body::from(assembled_body));

                    {
                        let headers = request.headers_mut();
                        headers.set(ContentType(mime::APPLICATION_WWW_FORM_URLENCODED));
                    }

                    let work = client.request(request).and_then(|res| {
                        res.body()
                            .for_each(|chunk| io::stdout().write_all(&chunk).map_err(From::from))
                    });

                    &handle.spawn(work.map_err(|_| ()));

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