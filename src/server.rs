//! SERVER MODULE

extern crate bus;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate mime;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate url;

use bus::{Bus, BusReader};
use futures::{Stream, future::Future};
use hyper::{Body, Chunk, Error, Method, StatusCode, Uri, client::{Client, HttpConnector},
            header::{ContentLength, ContentType}, server::{Request, Response, Service}};
use hyper_tls::HttpsConnector;
use serde_json::Value;
use std::{collections::HashMap, io::{self, prelude::*}, sync::{Arc, Mutex, mpsc::Sender}};
use tokio_core::reactor::Handle;
use url::form_urlencoded;

const GET_RESPONSE: &str = "This server expects POST requests";
const PAYLOAD: &str = "payload";
const CALLBACK_ID: &str = "callback_id";
const NUM_THREADS: usize = 4;

type UrlTupleSender = Arc<Mutex<Bus<(String, Option<String>)>>>;

/// SERVER STRUCT
pub struct Server {
    core_handle: Handle,
    response_to_slack: String,
    send_callback_id: Sender<String>,
    send_url_bus: UrlTupleSender,
}

type ResponseStream = Box<Stream<Item = Chunk, Error = Error>>;

impl Server {
    /// VERY NAYSU METHOD
    pub fn new(
        core_handle: Handle,
        response_to_slack: String,
        send_callback_id: Sender<String>,
        send_url_bus: UrlTupleSender,
    ) -> Self {
        Server {
            core_handle,
            response_to_slack,
            send_callback_id,
            send_url_bus,
        }
    }
}

impl Service for Server {
    type Request = Request;
    type Error = Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
    type Response = Response<ResponseStream>;

    fn call(&self, req: Request) -> Self::Future {
        let mut response = Response::new();

        match req.method() {
            Method::Post => {
                let handle = self.core_handle.clone();
                let response_to_slack = self.response_to_slack.clone();
                let send_callback_id = self.send_callback_id.clone();
                let send_url_bus = self.send_url_bus.clone();

                return Box::new(req.body().concat2().map(move |assembled_body| {
                    let parameter = form_urlencoded::parse(assembled_body.as_ref())
                        .into_owned()
                        .collect::<HashMap<String, String>>();

                    let parsed_payload: Value = if let Some(n) = parameter.get(PAYLOAD) {
                        serde_json::from_str(n).expect("Couldn't parse formurl into JSON")
                    } else {
                        let body: ResponseStream = Box::new(hyper::Body::from(GET_RESPONSE));
                        return Response::new()
                            .with_status(StatusCode::UnprocessableEntity)
                            .with_header(ContentLength(GET_RESPONSE.len() as u64))
                            .with_body(body);
                    };

                    let client = create_client(&handle);

                    let recv_url = { send_url_bus.lock().unwrap().add_rx() };
                    let callback_id_str = parsed_payload[CALLBACK_ID].as_str().unwrap();
                    let url = send_id_receive_url(&send_callback_id, callback_id_str, recv_url);

                    let request = new_request(
                        Method::Post,
                        url,
                        assembled_body,
                        mime::APPLICATION_WWW_FORM_URLENCODED,
                    );

                    let work = client.request(request).and_then(|res| {
                        res.body()
                            .for_each(|chunk| io::stdout().write_all(&chunk).map_err(From::from))
                    });

                    handle.spawn(work.map_err(|_| ()));
                    new_response(response_to_slack)
                }));
            }
            Method::Get => {
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

fn send_id_receive_url(
    send_callback_id: &Sender<String>,
    callback_id_str: &str,
    mut recv_url: BusReader<(String, Option<String>)>,
) -> Uri {
    send_callback_id.send(callback_id_str.to_string());
    let mut url_option = None;
    while let Ok(id_url_tuple) = recv_url.recv() {
        if callback_id_str == id_url_tuple.0 {
            url_option = id_url_tuple.1;
            break;
        }
    }
    url_option
        .expect("Requested callback_id has no mapping")
        .parse()
        .expect("Could not parse mapping into URL")
}

fn create_client(handle_ref: &Handle) -> Client<HttpsConnector<HttpConnector>, Body> {
    Client::configure()
        .connector(HttpsConnector::new(NUM_THREADS, handle_ref).unwrap())
        .build(handle_ref)
}

fn new_request(method: Method, url: Uri, body: Chunk, content_type: mime::Mime) -> Request {
    let mut request = Request::new(method, url);
    request.set_body(Body::from(body));
    {
        let headers = request.headers_mut();
        headers.set(ContentType(content_type));
    }
    request
}

fn new_response(response_to_slack: String) -> Response<ResponseStream> {
    let body = response_to_slack;
    let len = body.len();
    let body: ResponseStream = Box::new(hyper::Body::from(body));
    Response::new()
        .with_header(ContentLength(len as u64))
        .with_body(body)
}
