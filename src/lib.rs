#![deny(missing_docs)]

//! rson is a **fast** and **concurrent** Slack notification router built on top of **hyper**.
//!
//! It makes adding apps and integrations on the fly trivial, providing a single URL for Slack to send POST requests to. rson then takes care of parsing incoming requests and determining where they should be routed to, be that Google App Scripts, 3rd Party Software, or other custom APIs.
//!
//! Getting started is as easy as:
//!
//! ```
//!     let addr = "localhost:1337".parse().unwrap();
//!     let mapfile = Path::new("mappings.json");
//!     let slack_response = "{\"text\": \" ✅ Your request has been received!\"}";
//!     start_server(addr, mapfile, slack_response).unwrap()
//! ```

extern crate bus;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate url;
#[macro_use]
extern crate lazy_static;

pub mod hostmap;
pub mod server;

use bus::Bus;
use futures::{Stream, future::Future};
use hostmap::HostMap;
use hyper::server::Http;
use regex::Regex;
use server::Server;
use std::{thread, io::{self, BufReader, prelude::*}, path::Path,
          sync::{Arc, Mutex, mpsc::{self, Receiver, Sender}}};

const REGEXPR: &str = r#"^(ls)|(add)(?: "(\w+)")(?: "([a-zA-Z0-9:/.]+)")|(rm)(?: "(\w+)")$"#;
const NUM_OF_BUSREADER: usize = 10;

lazy_static! {
    static ref RE_COMMAND: Regex = Regex::new(REGEXPR).expect("Could not parse supplied regex!");
}

/// This enum represents a command that has been issued to
/// the [HostMap](hostmap/struct.HostMap.html) struct.
pub enum Command {
    /// This variant represents the command to add a new mapping
    /// to the [HostMap](hostmap/struct.HostMap.html) struct or to
    /// update an existing one. The first field is the callback id, and
    /// the second field is the corresponding url.
    Add(String, String),
    /// This variant represents the command to remove an existing mapping
    /// from the [HostMap](hostmap/struct.HostMap.html) struct. Its only
    /// field is the callback id to be removed.
    Remove(String),
    /// This variant represents the command to list (by pretty printing)
    /// all mappings found in the [HostMap](hostmap/struct.HostMap.html) struct.
    List,
}

/// BEST FUNCTION EVER
pub fn start_server(
    addr: std::net::SocketAddr,
    dict_file: &'static Path,
    response_to_slack: &str,
) -> hyper::Result<()> {
    let (send_callback_id, send_url_bus) = spawn_hostmap(dict_file);

    let mut core = tokio_core::reactor::Core::new()?;
    let server_handle = core.handle();
    let client_handle = core.handle();
    let response_to_slack = response_to_slack.to_string();
    let serve = Http::new().serve_addr_handle(&addr, &server_handle, move || {
        Ok(Server::new(
            client_handle.clone(),
            response_to_slack.clone(),
            send_callback_id.clone(),
            send_url_bus.clone(),
        ))
    })?;

    println!("Listening on http://{}", serve.incoming_ref().local_addr());
    let server_handle2 = server_handle.clone();
    server_handle.spawn(
        serve
            .for_each(move |conn| {
                server_handle2.spawn(
                    conn.map(|_| ())
                        .map_err(|err| println!("serve error: {:?}", err)),
                );
                Ok(())
            })
            .map_err(|_| ()),
    );
    core.run(futures::future::empty::<(), hyper::Error>())
}

type SenderTuple = (Sender<String>, Arc<Mutex<Bus<(String, Option<String>)>>>);

fn spawn_hostmap(host_file: &'static Path) -> SenderTuple {
    let (send_callback_id, recv_callback_id): (Sender<String>, Receiver<String>) = mpsc::channel();
    let send_url_bus = Arc::new(Mutex::new(Bus::new(NUM_OF_BUSREADER)));
    let send_url_bus_clone = send_url_bus.clone();
    let recv_callback_id = Arc::new(Mutex::new(recv_callback_id));

    thread::spawn(move || {
        let (send_new_line, recv_new_line): (Sender<Command>, Receiver<Command>) = mpsc::channel();

        thread::spawn(move || {
            BufReader::new(io::stdin()).lines().for_each(|new_line| {
                let new_line = new_line.expect("Could not read line");
                match_and_send(&new_line, &send_new_line);
            });
        });

        let hostmap = Arc::new(Mutex::new(HostMap::new_from_file(host_file).unwrap()));
        let recv_callback_id = recv_callback_id.clone();
        let hostmap_clone = hostmap.clone();

        thread::spawn(move || loop {
            let callback_id = recv_callback_id.lock().unwrap().recv().unwrap();
            let url = hostmap_clone.lock().unwrap().resolve_callback(&callback_id);
            send_url_bus_clone
                .lock()
                .unwrap()
                .broadcast((callback_id, url));
        });
        print_carets().unwrap();
        loop {
            if let Ok(new_command) = recv_new_line.recv() {
                let mut hostmap = hostmap.lock().unwrap();
                match new_command {
                    Command::List => {
                        println!("List of mappings:\n{:#?}", *hostmap);
                    }
                    Command::Add(callback_id, url) => {
                        println!("Mapping for {} has been inserted", callback_id);
                        hostmap.insert(callback_id, url);
                    }
                    Command::Remove(callback_id) => match hostmap.remove(&callback_id) {
                        Some(_) => {
                            println!("Mapping for {} has been removed", callback_id);
                        }
                        None => {
                            println!("Can't remove {} because it doesn't exist", callback_id);
                        }
                    },
                }
                print_carets().unwrap();
            }
        }
    });

    (send_callback_id, send_url_bus)
}

fn match_and_send(new_line: &str, send_new_line: &Sender<Command>) {
    let re_try = RE_COMMAND.captures(new_line);
    match re_try {
        Some(array) => {
            // Multiple nested `match`es because regex crate doesn't implement branch reset groups
            match array.get(1) {
                Some(_) => {
                    send_new_line.send(Command::List);
                }
                None => match array.get(2) {
                    Some(_) => {
                        send_new_line
                            .send(Command::Add(array[3].to_string(), array[4].to_string()));
                    }
                    None => match array.get(5) {
                        Some(_) => {
                            send_new_line.send(Command::Remove(array[6].to_string()));
                        }
                        None => unreachable!(),
                    },
                },
            }
        }
        None => {
            println!("Command not recognized");
        }
    }
}

fn print_carets() -> io::Result<()> {
    print!(">>> ");
    io::stdout().flush()
}
