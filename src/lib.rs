extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate url;

#[macro_use]
extern crate serde_derive;

use futures::{Stream, future::Future};
use hyper::{Body, Chunk, Error, Method, StatusCode, header::ContentLength,
            server::{Http, Request, Response, Service}};
use serde_json::{Error as SerdeError, Value};
use std::{collections::HashMap, fs::{File, OpenOptions}, io::{self, prelude::*}, path::Path};
use url::form_urlencoded;

const GET_RESPONSE: &'static str = "This server expects POST requests to /";
static MISSING: &[u8] = b"Missing field";
const NUM_THREADS: usize = 4;

pub struct SimpleRespond(tokio_core::reactor::Handle, Dictionary);

// For extra client
pub type ResponseStream = Box<Stream<Item = Chunk, Error = Error>>;

impl Service for SimpleRespond {
    type Request = Request;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;
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
                    let client = ::hyper::Client::configure()
                        .connector(::hyper_tls::HttpsConnector::new(4, &handle).unwrap())
                        .build(&handle);
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

fn resolve_callback(mut id: &serde_json::Value) -> serde_json::Value {
    let mut mapfile = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("mappings.json")
        .expect("Mappings file IO error!");

    let json: Value =
        serde_json::from_reader(mapfile).expect("Couldn't read file into JSON Object!");

    json[id.as_str().unwrap()].clone()
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dictionary {
    mappings: HashMap<String, String>,
}

impl Dictionary {
    pub fn new_from_file(path: &Path) -> io::Result<Self> {
        let mut mapfile = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)
            .expect("Mapfile IO error!");

        let mappings: HashMap<String, String> =
            serde_json::from_reader(mapfile).expect("Couldn't read file into JSON Object!");

        println!("{:?}", mappings);
        Ok(Dictionary { mappings })
    }
    pub fn resolve_callback(&self, id: &str) -> Option<&String> {
        self.mappings.get(id)
    }
}

pub fn start_server(addr: std::net::SocketAddr, dict_file: &Path) -> hyper::Result<()> {
    let dictionary = Dictionary::new_from_file(dict_file).unwrap();

    let mut core = tokio_core::reactor::Core::new()?;
    let server_handle = core.handle();
    let client_handle = core.handle();
    let serve = Http::new().serve_addr_handle(&addr, &server_handle, move || {
        Ok(SimpleRespond(client_handle.clone(), dictionary))
    })?;

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
    core.run(futures::future::empty::<(), hyper::Error>())
}

fn print_carets() -> io::Result<()> {
    print!(">>> ");
    io::stdout().flush()
}
